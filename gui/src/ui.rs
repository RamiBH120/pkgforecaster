use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, Button, Label, ListBox, Orientation, Paned, ScrolledWindow,
    TextView, DrawingArea, gdk, cairo,
};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::process::Command;
use glib::MainContext;

#[derive(Debug, Deserialize, Clone)]
pub struct PackageUpdate {
    pub name: String,
    pub current: String,
    pub new: String,
    pub risks: Vec<String>,
    // add a numeric risk score for heatmap
    pub risk_score: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Simulation {
    pub updates: Vec<PackageUpdate>,
    pub summary: Summary,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Summary {
    pub total: u32,
    pub high_risk: u32,
    pub medium_risk: u32,
}

pub fn run() -> anyhow::Result<()> {
    let app = gtk4::Application::new(
        Some("com.rami.pkgforecaster"),
        Default::default(),
    );

    app.connect_activate(move |app| {
        let win = ApplicationWindow::new(app);
        win.set_title(Some("PkgForecaster"));
        win.set_default_size(1200, 800);

        let hpaned = Paned::new(Orientation::Horizontal);

        // Sidebar
        let sidebar_box = GtkBox::new(Orientation::Vertical, 8);
        sidebar_box.set_margin_all(12);
        let header = Label::new(Some("PkgForecaster"));
        header.set_halign(gtk4::Align::Start);
        sidebar_box.append(&header);
        let scan_button = Button::with_label("Run Simulation");
        sidebar_box.append(&scan_button);
        sidebar_box.append(&Label::new(Some("Recent Simulations")));
        let listbox = ListBox::new();
        sidebar_box.append(&listbox);
        listbox.append(&Label::new(Some("simulation-dummy")));
        let sidebar_scroll = ScrolledWindow::new();
        sidebar_scroll.set_child(Some(&sidebar_box));
        hpaned.add1(&sidebar_scroll);

        // Main content
        let main_vbox = GtkBox::new(Orientation::Vertical, 6);
        main_vbox.set_margin_all(8);
        let tab_bar = GtkBox::new(Orientation::Horizontal, 6);
        let _overview_btn = Button::with_label("Overview");
        let _details_btn = Button::with_label("Details");
        tab_bar.append(&_overview_btn);
        tab_bar.append(&_details_btn);
        main_vbox.append(&tab_bar);

        let content_paned = Paned::new(Orientation::Horizontal);
        let left_box = GtkBox::new(Orientation::Vertical, 6);
        left_box.set_margin_all(6);
        left_box.append(&Label::new(Some("Packages to update")));
        let pkg_list = ListBox::new();
        pkg_list.set_vexpand(true);
        left_box.append(&pkg_list);

        let right_vbox = GtkBox::new(Orientation::Vertical, 6);
        right_vbox.set_margin_all(6);
        let diff_label = Label::new(Some("Diff Viewer"));
        right_vbox.append(&diff_label);
        let diff_view = TextView::new();
        diff_view.set_wrap_mode(gtk4::pango::WrapMode::Word);
        diff_view.set_vexpand(true);
        right_vbox.append(&diff_view);

        // Heatmap DrawingArea
        let heatmap_area = DrawingArea::new();
        heatmap_area.set_content_width(400);
        heatmap_area.set_content_height(300);
        heatmap_area.set_vexpand(false);
        heatmap_area.set_hexpand(true);

        // shared simulation state for the UI thread
        let sim_state: Arc<Mutex<Option<Simulation>>> = Arc::new(Mutex::new(None));
        let sim_state_clone = sim_state.clone();

        // painting closure: draw heatmap grid based on simulation state
        heatmap_area.set_draw_func(move |area, ctx, width, height| {
            let guard = sim_state_clone.lock().unwrap();
            let sim = guard.clone();

            // clear background
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.paint().unwrap();

            // if no sim data, show placeholder text
            if sim.is_none() {
                ctx.set_source_rgb(0.2, 0.2, 0.2);
                ctx.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                ctx.set_font_size(14.0);
                ctx.move_to(20.0, (height / 2) as f64);
                ctx.show_text("Risk Heatmap: run simulation to populate").unwrap();
                return;
            }

            let sim = sim.unwrap();
            let n = sim.updates.len().max(1);
            // grid dimensions: try a near-square layout
            let cols = (f64::sqrt(n as f64)).ceil() as usize;
            let rows = ((n + cols - 1) / cols) as usize;
            let pad = 8.0;
            let cell_w = (width as f64 - pad * 2.0) / (cols as f64);
            let cell_h = (height as f64 - pad * 2.0) / (rows as f64);

            // compute min/max risk_score
            let mut min_score = f64::INFINITY;
            let mut max_score = f64::NEG_INFINITY;
            for p in sim.updates.iter() {
                let s = p.risk_score.unwrap_or(0.0);
                if s < min_score { min_score = s; }
                if s > max_score { max_score = s; }
            }
            if min_score.is_infinite() || max_score.is_infinite() {
                min_score = 0.0; max_score = 1.0;
            }
            let range = (max_score - min_score).max(1e-6);

            for (i, p) in sim.updates.iter().enumerate() {
                let r = i / cols;
                let c = i % cols;
                let x = pad + (c as f64) * cell_w;
                let y = pad + (r as f64) * cell_h;
                let score = p.risk_score.unwrap_or(0.0);
                let t = (score - min_score) / range; // 0..1

                // color map: green (low) -> yellow -> red (high)
                let (rcol, gcol, bcol) = if t < 0.5 {
                    let t2 = t / 0.5;
                    (t2, 1.0, 0.0)
                } else {
                    let t2 = (t - 0.5) / 0.5;
                    (1.0, 1.0 - t2, 0.0)
                };
                ctx.set_source_rgb(rcol, gcol, bcol);
                ctx.rectangle(x, y, cell_w - 4.0, cell_h - 4.0);
                ctx.fill().unwrap();

                // draw label
                ctx.set_source_rgb(0.0, 0.0, 0.0);
                ctx.set_font_size(10.0);
                let label = format!("{} ({:.2})", p.name, score);
                ctx.move_to(x + 6.0, y + 14.0);
                ctx.show_text(&label).unwrap();
            }
        });

        right_vbox.append(&heatmap_area);
        content_paned.add1(&left_box);
        content_paned.add2(&right_vbox);
        main_vbox.append(&content_paned);
        hpaned.add2(&main_vbox);

        // initial dummy load
        if let Ok(sim) = load_dummy_simulation("data/dummy_simulation.json") {
            // ensure risk_score exists; fill stub scores if missing
            let mut sim = sim;
            for (i, mut p) in sim.updates.iter_mut().enumerate() {
                if p.risk_score.is_none() {
                    p.risk_score = Some(((i as f64) % 10.0) / 10.0); // sample spread
                }
            }
            // populate UI
            for p in sim.updates.iter() {
                let row_label = Label::new(Some(&format!("{}  {}→{}", p.name, p.current, p.new)));
                pkg_list.append(&row_label);
            }
            // store in shared state
            *sim_state.lock().unwrap() = Some(sim);
        }

        // wire scan button to run engine asynchronously and update UI
        let diff_view_clone = diff_view.clone();
        let sim_state_for_thread = sim_state.clone();
        scan_button.connect_clicked(move |_| {
            // Run engine CLI asynchronously on a thread, then schedule update on main loop
            std::thread::spawn(move || {
                // Try to call engine CLI; if fails, fallback to bundled dummy file
                let output = Command::new("../engine/target/release/engine-cli")
                    .arg("--apt-sim")
                    .output();

                let stdout = match output {
                    Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
                    _ => {
                        // fallback: read data/dummy_simulation.json plain (JSON already shaped)
                        std::fs::read_to_string("data/dummy_simulation.json").unwrap_or_default()
                    }
                };

                // parse JSON into Simulation
                let parsed: Result<Simulation, _> = serde_json::from_str(&stdout);
                if let Ok(mut sim) = parsed {
                    // ensure risk scores exist
                    for (i, mut p) in sim.updates.iter_mut().enumerate() {
                        if p.risk_score.is_none() {
                            p.risk_score = Some(((i as f64) % 10.0) / 10.0);
                        }
                    }
                    // schedule UI update on main thread
                    let ctx = MainContext::default();
                    ctx.spawn_local(async move {
                        // store into sim_state and queue redraw
                        *sim_state_for_thread.lock().unwrap() = Some(sim.clone());
                        diff_view_clone.buffer().set_text(&serde_json::to_string_pretty(&sim).unwrap());
                        heatmap_area.queue_draw();
                    });
                } else {
                    // parse failed, write raw stdout to diff_view
                    let ctx = MainContext::default();
                    ctx.spawn_local(async move {
                        diff_view_clone.buffer().set_text(&stdout);
                    });
                }
            });
        });

        win.set_child(Some(&hpaned));
        win.show();
    });

    app.run();
    Ok(())
}

/// Load dummy simulation JSON
fn load_dummy_simulation<P: AsRef<Path>>(path: P) -> anyhow::Result<Simulation> {
    let mut s = String::new();
    let mut f = File::open(path)?;
    f.read_to_string(&mut s)?;
    let sim: Simulation = serde_json::from_str(&s)?;
    Ok(sim)
}
