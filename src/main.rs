use gtk::{gdk, gio, glib, graphene, prelude::*, subclass::prelude::*};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

mod frame {
    use super::*;
    #[derive(Default)]
    pub struct Frame {
        pub media: RefCell<Option<gtk::MediaFile>>,
        pub inverted: Cell<bool>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for Frame {
        const NAME: &'static str = "HermitVideoFrame";
        type Type = super::Frame;
        type ParentType = gtk::Widget;
    }
    impl ObjectImpl for Frame {}
    impl WidgetImpl for Frame {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if let Some(media) = self.media.borrow().as_ref() {
                let width = gtk::prelude::WidgetExt::width(&*self.obj()) as f64;
                let height = gtk::prelude::WidgetExt::height(&*self.obj()) as f64;
                if self.inverted.get() {
                    let matrix = graphene::Matrix::from_float([
                        -1., 0., 0., 0., 0., -1., 0., 0., 0., 0., -1., 0., 0., 0., 0., 1.,
                    ]);
                    snapshot.push_color_matrix(&matrix, &graphene::Vec4::new(1., 1., 1., 0.));
                }
                let pw = media.intrinsic_width().max(1) as f64;
                let ph = media.intrinsic_height().max(1) as f64;
                let scale = (width / pw).min(height / ph);
                let (w, h) = (pw * scale, ph * scale);
                snapshot.save();
                snapshot.translate(&graphene::Point::new(
                    ((width - w) / 2.) as f32,
                    ((height - h) / 2.) as f32,
                ));
                media.snapshot(snapshot, w, h);
                snapshot.restore();
                if self.inverted.get() {
                    snapshot.pop();
                }
            }
        }
    }
}
glib::wrapper! { pub struct Frame(ObjectSubclass<frame::Frame>) @extends gtk::Widget, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget; }
impl Frame {
    fn new(media: &gtk::MediaFile) -> Self {
        let frame: Self = glib::Object::new();
        *frame.imp().media.borrow_mut() = Some(media.clone());
        frame.set_hexpand(true);
        frame.set_vexpand(true);
        let weak = frame.downgrade();
        media.connect_invalidate_contents(move |_| {
            if let Some(frame) = weak.upgrade() {
                frame.queue_draw();
            }
        });
        frame
    }
    fn flash(&self) {
        self.imp().inverted.set(true);
        self.queue_draw();
        let weak = self.downgrade();
        glib::timeout_add_local_once(Duration::from_millis(650), move || {
            if let Some(frame) = weak.upgrade() {
                frame.imp().inverted.set(false);
                frame.queue_draw();
            }
        });
    }
}
fn sound() -> Option<gtk::MediaFile> {
    let path = std::env::temp_dir().join(format!("hermit-pause-{}.wav", std::process::id()));
    std::fs::write(&path, include_bytes!("../assets/pause.wav")).ok()?;
    Some(gtk::MediaFile::for_filename(path))
}
fn toggle(media: &gtk::MediaFile, frame: &Frame, effect: &Option<gtk::MediaFile>) {
    if !media.is_prepared() {
        return;
    }
    if media.is_playing() {
        media.pause();
        frame.flash();
        if let Some(effect) = effect {
            effect.seek(0);
            effect.play();
        }
    } else {
        media.play();
    }
}
fn open_file(media: &gtk::MediaFile, label: &gtk::Label, file: &gio::File) {
    media.set_file(Some(file));
    label.set_text(
        &file
            .basename()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Video".into()),
    );
    media.play();
}
fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("site.blig.Hermit")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    app.connect_activate(|app| build(app, None));
    app.connect_open(|app, files, _| build(app, files.first()));
    app.connect_shutdown(|_| {
        let path = std::env::temp_dir().join(format!("hermit-pause-{}.wav", std::process::id()));
        let _ = std::fs::remove_file(path);
    });
    app.run()
}
fn build(app: &gtk::Application, file: Option<&gio::File>) {
    let media = gtk::MediaFile::new();
    let frame = Frame::new(&media);
    let effect = Rc::new(sound());
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Hermit")
        .default_width(1000)
        .default_height(650)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = gtk::HeaderBar::new();
    let open = gtk::Button::with_label("Open");
    header.pack_start(&open);
    let label = gtk::Label::new(Some("Open a video"));
    label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    header.set_title_widget(Some(&label));
    window.set_titlebar(Some(&header));
    frame.add_css_class("video-frame");
    root.append(&frame);
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    controls.set_margin_top(12);
    controls.set_margin_bottom(12);
    controls.set_margin_start(16);
    controls.set_margin_end(16);
    let play = gtk::Button::with_label("Play");
    controls.append(&play);
    let time = gtk::Label::new(Some("0:00"));
    controls.append(&time);
    let progress = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0., 1., 1.);
    progress.set_draw_value(false);
    progress.set_hexpand(true);
    progress.set_sensitive(false);
    controls.append(&progress);
    let duration = gtk::Label::new(Some("0:00"));
    controls.append(&duration);
    let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0., 1., 0.05);
    volume.set_value(1.);
    volume.set_draw_value(false);
    volume.set_width_request(90);
    volume.set_tooltip_text(Some("Volume"));
    controls.append(&volume);
    root.append(&controls);
    window.set_child(Some(&root));
    let css = gtk::CssProvider::new();
    css.load_from_data(".video-frame { background: #0d0d10; }");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    let m = media.clone();
    let f = frame.clone();
    let e = effect.clone();
    play.connect_clicked(move |_| toggle(&m, &f, &e));
    let p = play.clone();
    media.connect_playing_notify(move |m| {
        p.set_label(if m.is_playing() { "Pause" } else { "Play" })
    });
    let updating = Rc::new(Cell::new(false));
    let u = updating.clone();
    let m = media.clone();
    progress.connect_value_changed(move |s| {
        if !u.get() && m.is_seekable() {
            m.seek(s.value() as i64);
        }
    });
    let p = progress.clone();
    let t = time.clone();
    let d = duration.clone();
    let u = updating.clone();
    media.connect_timestamp_notify(move |m| {
        u.set(true);
        let total = m.duration().max(0);
        p.set_range(0., total.max(1) as f64);
        p.set_sensitive(m.is_seekable());
        p.set_value(m.timestamp().max(0) as f64);
        t.set_text(&clock(m.timestamp()));
        d.set_text(&clock(total));
        u.set(false);
    });
    let m = media.clone();
    volume.connect_value_changed(move |v| m.set_volume(v.value()));
    let w = window.downgrade();
    let m = media.clone();
    let l = label.clone();
    open.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            let dialog = gtk::FileChooserNative::new(
                Some("Open video"),
                Some(&w),
                gtk::FileChooserAction::Open,
                Some("Open"),
                Some("Cancel"),
            );
            let m = m.clone();
            let l = l.clone();
            dialog.connect_response(move |d, r| {
                if r == gtk::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        open_file(&m, &l, &file);
                    }
                }
                d.destroy();
            });
            dialog.show();
        }
    });
    let m = media.clone();
    let f = frame.clone();
    let e = effect.clone();
    let keys = gtk::EventControllerKey::new();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::space {
            toggle(&m, &f, &e);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(keys);
    let l = label.clone();
    media.connect_error_notify(move |m| {
        if let Some(e) = m.error() {
            l.set_text(&format!("Could not play video: {e}"));
        }
    });
    if let Some(file) = file {
        open_file(&media, &label, file);
    }
    window.present();
}
fn clock(micros: i64) -> String {
    let seconds = micros.max(0) / 1_000_000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
