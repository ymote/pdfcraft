//! Vector icons: embedded Lucide SVGs, recoloured to white, rasterized by egui_extras at the exact
//! on-screen pixel size, and tinted per use (PhotoCraft lesson: never use Unicode glyphs as icons).

use std::sync::OnceLock;

use craft_ui::buttons::{ButtonVisuals, IconButton, IconButtonStyle};
use craft_ui::icons::{SvgIconSet, SvgStyle};

use egui::{Color32, Rect, Response, Vec2};

use crate::icon_data::ICONS;
use crate::theme::Tokens;

fn white_icons() -> &'static SvgIconSet {
    static MAP: OnceLock<SvgIconSet> = OnceLock::new();
    MAP.get_or_init(|| SvgIconSet::new(ICONS, SvgStyle { current_color: "#ffffff", stroke_width: Some("1.75") }, "bytes://icons/", "square"))
}

pub fn exists(name: &str) -> bool {
    white_icons().contains(name)
}

fn source(name: &str) -> egui::ImageSource<'static> {
    // PdfCraft caches an unknown name under the resolved fallback name.
    white_icons().source(white_icons().canonical_name(name).unwrap_or("square"))
}

pub fn image(name: &str, size: f32, tint: Color32) -> egui::Image<'static> {
    egui::Image::new(source(name)).fit_to_exact_size(Vec2::splat(size)).tint(tint)
}

pub fn paint(ui: &egui::Ui, rect: Rect, name: &str, size: f32, tint: Color32) {
    image(name, size, tint).paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(size)));
}

/// Square icon button: transparent until hovered; `selected` gets the accent treatment.
pub fn button(ui: &mut egui::Ui, name: &str, box_size: f32, selected: bool, tooltip: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let style = IconButtonStyle {
        normal: ButtonVisuals { fill: Color32::TRANSPARENT, stroke: egui::Stroke::NONE, icon: t.icon },
        hovered: ButtonVisuals { fill: t.hover, stroke: egui::Stroke::NONE, icon: t.icon },
        pressed: Some(ButtonVisuals { fill: t.pressed, stroke: egui::Stroke::NONE, icon: t.icon }),
        selected: ButtonVisuals { fill: t.accent_soft, stroke: egui::Stroke::NONE, icon: t.accent_text },
        corner_radius: t.radius.into(),
        focus_stroke: egui::Stroke::new(1.0, t.accent),
    };
    let label = if tooltip.is_empty() { name } else { tooltip };
    let resp = IconButton::new(label, Vec2::splat(box_size), &style).selected(selected).show(ui, |ui, rect, tint| {
        paint(ui, rect, name, (box_size * 0.5).round(), tint);
    });
    if tooltip.is_empty() { resp } else { resp.on_hover_text(tooltip) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_catalog_icon_exists() {
        for g in pdfcraft_engine::catalog::TOOL_GROUPS {
            assert!(super::exists(g.icon), "missing icon {}", g.icon);
            for s in g.sections {
                for i in s.items {
                    assert!(super::exists(i.icon), "missing icon {}", i.icon);
                }
            }
        }
    }

    #[test]
    fn every_command_icon_exists() {
        for c in pdfcraft_engine::commands::COMMANDS {
            assert!(super::exists(c.icon), "missing icon {} ({})", c.icon, c.id);
        }
    }

    #[test]
    fn shared_catalog_preserves_existing_icons_and_fallback_uri() {
        assert_eq!(ICONS.len(), 185);
        assert!(exists("minimize-2"));
        for (name, original) in ICONS {
            let expected =
                String::from_utf8_lossy(original).replace("currentColor", "#ffffff").replace("stroke-width=\"2\"", "stroke-width=\"1.75\"");
            let egui::ImageSource::Bytes { uri, bytes } = source(name) else { panic!("expected SVG bytes") };
            assert_eq!(uri.as_ref(), format!("bytes://icons/{name}.svg"));
            assert_eq!(bytes.as_ref(), expected.as_bytes(), "{name}");
        }
        let egui::ImageSource::Bytes { uri, bytes } = source("missing-icon") else { panic!("expected SVG bytes") };
        let egui::ImageSource::Bytes { bytes: fallback, .. } = source("square") else { panic!("expected SVG bytes") };
        assert_eq!(uri.as_ref(), "bytes://icons/square.svg");
        assert_eq!(bytes.as_ref(), fallback.as_ref());
        assert!(!exists("missing-icon"));
    }

    #[test]
    fn shared_button_preserves_theme_painting_and_pointer_responses() {
        // Freeze the original paint path across both themes. The added keyboard-focus
        // outline is covered independently in craft-ui.
        fn frame(
            ctx: &egui::Context,
            events: Vec<egui::Event>,
            selected: bool,
            enabled: bool,
            legacy: bool,
        ) -> (egui::Response, Vec<egui::epaint::ClippedShape>) {
            let mut response = None;
            let mut output = ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
                response = Some(
                    ui.add_enabled_ui(enabled, |ui| {
                        if legacy {
                            let t = Tokens::get(ui.ctx());
                            let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
                            if selected {
                                ui.painter().rect_filled(rect, t.radius, t.accent_soft);
                            } else if response.is_pointer_button_down_on() {
                                ui.painter().rect_filled(rect, t.radius, t.pressed);
                            } else if response.hovered() {
                                ui.painter().rect_filled(rect, t.radius, t.hover);
                            }
                            let tint = if selected { t.accent_text } else { t.icon };
                            paint(ui, rect, "square", (28.0_f32 * 0.5).round(), tint);
                            response
                        } else {
                            button(ui, "square", 28.0, selected, "")
                        }
                    })
                    .inner,
                );
            });
            output.textures_delta.clear();
            (response.unwrap(), output.shapes)
        }
        for theme in [crate::theme::ThemeKind::Dark, crate::theme::ThemeKind::Light] {
            for selected in [false, true] {
                for enabled in [false, true] {
                    let old = egui::Context::default();
                    let new = egui::Context::default();
                    for ctx in [&old, &new] {
                        crate::theme::apply(ctx, theme);
                        egui_extras::install_image_loaders(ctx);
                    }
                    let mut center = egui::Pos2::ZERO;
                    for step in 0..7 {
                        let events = match step {
                            2 => vec![egui::Event::PointerMoved(center)],
                            3 | 4 => vec![egui::Event::PointerButton {
                                pos: center,
                                button: egui::PointerButton::Primary,
                                pressed: step == 3,
                                modifiers: egui::Modifiers::NONE,
                            }],
                            5 => vec![egui::Event::PointerMoved(egui::pos2(300.0, 200.0))],
                            _ => vec![],
                        };
                        let (a, before) = frame(&old, events.clone(), selected, enabled, true);
                        let (b, after) = frame(&new, events, selected, enabled, false);
                        center = a.rect.center();
                        assert_eq!(a.rect, b.rect);
                        assert_eq!(
                            (a.clicked(), a.hovered(), a.is_pointer_button_down_on()),
                            (b.clicked(), b.hovered(), b.is_pointer_button_down_on())
                        );
                        assert_eq!(before, after, "{theme:?}, selected={selected}, enabled={enabled}, step={step}");
                    }
                }
            }
        }
    }
}
