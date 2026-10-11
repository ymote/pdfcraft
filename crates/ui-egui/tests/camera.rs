//! Scan & OCR ▸ Enhance ▸ Camera image in the real shell (egui_kittest): drag a corner, then
//! Enhance page; Cancel and Esc leave the page as it was.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::{PdfCraftApp, QuickTool};

const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 /MediaBox [0 0 300 400] >> endobj
3 0 obj << /Type /Page /Parent 2 0 R >> endobj
4 0 obj << /Type /Page /Parent 2 0 R >> endobj
trailer << /Root 1 0 R >>
%%EOF";

fn harness() -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, FIXTURE.to_vec()).unwrap();
        app.set_option("left", "closed").unwrap();
        app.set_option("zoom", "100").unwrap();
        app
    });
    h.run_steps(6);
    h
}

fn size(h: &Harness<'static, PdfCraftApp>, page: usize) -> (f32, f32) {
    let s = h.state();
    let p = &s.session.get(s.views[0].id).unwrap().info.pages[page];
    (p.width, p.height)
}

fn undo_label(h: &Harness<'static, PdfCraftApp>) -> Option<String> {
    let s = h.state();
    s.session.get(s.views[0].id).unwrap().can_undo().map(str::to_owned)
}

#[test]
fn dragging_a_corner_then_enhance_page_straightens_the_page() {
    let mut h = harness();
    assert!(h.state_mut().execute("ocr.enhance_camera"));
    assert_eq!(h.state().quick_tool, QuickTool::EnhanceCamera);
    h.run_steps(3);
    let quad = h.state().views[0].camera.expect("the corners start on the current page");
    assert_eq!(quad.page, 0);
    assert_eq!(quad.corners, [[15.0, 20.0], [285.0, 20.0], [285.0, 380.0], [15.0, 380.0]], "inset by 5%");

    // Drag the top-left corner to (60, 80) on the page.
    let r = h.state().views[0].page_screen_rect(0).unwrap();
    let (a, b) = (r.min + r.size() * 0.05, r.min + r.size() * 0.2);
    h.hover_at(a);
    h.run_steps(1);
    h.drag_at(a);
    h.run_steps(1);
    for k in 1..=4 {
        h.hover_at(a + (b - a) * (k as f32 / 4.0));
        h.run_steps(1);
    }
    h.drop_at(b);
    h.run_steps(2);
    let c = h.state().views[0].camera.unwrap().corners[0];
    assert!((c[0] - 60.0).abs() < 1.0 && (c[1] - 80.0).abs() < 1.0, "{c:?}");
    assert_eq!(undo_label(&h), None, "dragging changes nothing yet");

    h.get_by_label("Enhance page").click();
    h.run_steps(4);
    // The longer of each pair of opposite sides: 270 × 360 pt.
    let (w, hh) = size(&h, 0);
    assert!((w - 270.0).abs() < 2.0 && (hh - 360.0).abs() < 2.0, "{w}×{hh}");
    assert_eq!(size(&h, 1), (300.0, 400.0), "only the page with the corners changes");
    assert_eq!(undo_label(&h).as_deref(), Some("Enhance camera image"));
    // The tool stays open for the enhanced page: turn it, then Done.
    assert_eq!(h.state().quick_tool, QuickTool::EnhanceCamera);
    assert!(h.state().views[0].camera.is_none());
    assert_eq!(h.state().views[0].camera_done, Some(0));
    h.get_by_label("Rotate clockwise").click();
    h.run_steps(4);
    let (w, hh) = size(&h, 0);
    assert!((w - 360.0).abs() < 2.0 && (hh - 270.0).abs() < 2.0, "{w}×{hh}");
    assert_eq!(undo_label(&h).as_deref(), Some("Rotate page"));
    h.get_by_label("Done").click();
    h.run_steps(3);
    assert_eq!(h.state().quick_tool, QuickTool::Select);
    assert_eq!(h.state().views[0].camera_done, None);
}

#[test]
fn after_enhance_page_crop_and_organize_take_over() {
    for (button, then) in [("Crop pages", QuickTool::Crop), ("Organize pages", QuickTool::Select)] {
        let mut h = harness();
        assert!(h.state_mut().execute("ocr.enhance_camera"));
        h.run_steps(3);
        h.get_by_label("Enhance page").click();
        h.run_steps(4);
        assert_eq!(h.state().views[0].camera_done, Some(0));
        h.get_by_label(button).click();
        h.run_steps(3);
        assert_eq!(h.state().quick_tool, then, "{button}");
        assert_eq!(h.state().views[0].camera_done, None, "{button}");
        assert_eq!(h.state().views[0].organize, button == "Organize pages");
    }
}

#[test]
fn cancel_and_escape_leave_the_page_alone() {
    let mut h = harness();
    assert!(h.state_mut().execute("ocr.enhance_camera"));
    h.run_steps(3);
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert_eq!(h.state().quick_tool, QuickTool::Select);
    assert!(h.state().views[0].camera.is_none());

    assert!(h.state_mut().execute("ocr.enhance_camera"));
    h.run_steps(3);
    assert!(h.state().views[0].camera.is_some());
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_eq!(h.state().quick_tool, QuickTool::Select);
    assert!(h.state().views[0].camera.is_none());
    assert_eq!(size(&h, 0), (300.0, 400.0));
    assert_eq!(undo_label(&h), None);
}
