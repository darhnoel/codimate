use crate::{style::*, ItemId, SlotId, SwapABEvent, SwapABMotion, SwapABTiming, SwapABTrace};
use codimate::*;
use codimate::{animation, sequence, Animation, Playable};
use codimate::{box_at, box_in, centered_text, Viewport};

const VIEW_W: f32 = 960.0;
const VIEW_H: f32 = 540.0;
const SLOT_W: f32 = 130.0;
const SLOT_H: f32 = 86.0;
const SLOT_GAP: f32 = 42.0;
const GROUP_W: f32 = SLOT_W * 2.0 + SLOT_GAP;
const GROUP_H: f32 = SLOT_H;
const SLOT_R: f32 = 8.0;
const ITEM_W: f32 = 104.0;
const ITEM_H: f32 = 60.0;
const ITEM_R: f32 = 8.0;
const HEADER_LINE_H: f32 = 35.0;
const HEADER_GAP: f32 = 118.0;
const HEADER_H: f32 = HEADER_LINE_H * 2.0;
const SLOT_LABEL_Y_OFFSET: f32 = 22.0;
const SLOT_LABEL_H: f32 = 24.0;
const BOTTOM_GAP: f32 = 58.0;
const BOTTOM_LINE_H: f32 = 34.0;
const BOTTOM_H: f32 = BOTTOM_LINE_H * 2.0;

#[derive(Clone, Copy)]
pub struct SwapABView;

pub fn swap_a_b_view() -> SwapABView {
    SwapABView
}

pub(crate) fn build_swap_a_b(
    name: &'static str,
    trace: SwapABTrace,
    motion: SwapABMotion,
    timing: SwapABTiming,
) -> (Box<dyn Playable>, Viewport) {
    let animations: Vec<Animation> = vec![
        animation(
            "initial",
            timing.intro,
            static_scene(
                "Initial mapping: A -> slot 0, B -> slot 1",
                trace.initial,
                None,
            ),
        ),
        animation("swap", timing.swap, swap_scene(&trace.swap, motion)),
        animation(
            "done",
            timing.done,
            static_scene(
                "After swap: B -> slot 0, A -> slot 1",
                trace.final_mapping,
                None,
            ),
        ),
    ];

    (
        Box::new(sequence(name, animations)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn main_group_slot() -> codimate::Slot {
    Viewport::new(VIEW_W, VIEW_H)
        .slot()
        .centered_child(Vec2::new(GROUP_W, GROUP_H))
}

fn slots() -> Vec<codimate::Slot> {
    main_group_slot().row(Vec2::new(SLOT_W, SLOT_H), SLOT_GAP, 2)
}

fn header_text_slots() -> Vec<codimate::Slot> {
    let header = main_group_slot().above(Vec2::new(VIEW_W, HEADER_H), HEADER_GAP);

    header.column(Vec2::new(VIEW_W, HEADER_LINE_H), 0.0, 2)
}

fn bottom_text_slots() -> Vec<codimate::Slot> {
    let bottom = main_group_slot().below(Vec2::new(VIEW_W, BOTTOM_H), BOTTOM_GAP);

    bottom.column(Vec2::new(VIEW_W, BOTTOM_LINE_H), 0.0, 2)
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn background(mut sc: Scene) -> Scene {
    let text_slots = header_text_slots();
    sc = sc.add(primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H)).style(style(BG, 0.0, BG)));
    sc = sc.add(centered_text(&text_slots[0], "Swap A B", 32.0, INK));
    sc.add(centered_text(
        &text_slots[1],
        "items have identity; layout slots stay fixed",
        16.0,
        MUTED,
    ))
}

fn add_slots(mut sc: Scene, active: Option<(SlotId, SlotId)>) -> Scene {
    let slots = slots();
    for slot in [SlotId::Left, SlotId::Right] {
        let visual_slot = &slots[slot.index()];
        let label_slot = visual_slot.below(Vec2::new(SLOT_W, SLOT_LABEL_H), SLOT_LABEL_Y_OFFSET);
        let label_text = slot.label();
        let stroke = if active.is_some_and(|(a, b)| a == slot || b == slot) {
            ACTIVE
        } else {
            SLOT_STROKE
        };
        sc = sc.add(
            box_in(visual_slot)
                .radius(SLOT_R)
                .style(style(SLOT_FILL, 1.8, stroke)),
        );
        sc = sc.add(centered_text(&label_slot, label_text, 15.0, SLOT_TEXT));
    }
    sc
}

fn item_color(item: ItemId) -> Color {
    match item {
        ItemId::A => ITEM_A,
        ItemId::B => ITEM_B,
    }
}

fn add_item(
    mut sc: Scene,
    item: ItemId,
    center: impl IntoAnimated<Vec2>,
    stroke: impl IntoAnimated<Color>,
    stroke_width: impl IntoAnimated<f32>,
) -> Scene {
    let center = center.into_animated();
    let text_x = center.clone();
    let text_y = center.clone();
    let label = item.label();
    sc = sc.add(
        box_at(center, Vec2::new(ITEM_W, ITEM_H))
            .radius(ITEM_R)
            .style(style(item_color(item), 1.8, INK))
            .stroke(stroke_width, stroke),
    );
    sc.add(
        text()
            .x(Animated::new(move |t| text_x.resolve(t).x))
            .y(Animated::new(move |t| text_y.resolve(t).y + 10.0))
            .text(label.to_string())
            .font_size(30.0)
            .fill(INK)
            .align(TextAlign::Center),
    )
}

fn add_mapping(mut sc: Scene, mapping: [(ItemId, SlotId); 2]) -> Scene {
    let slots = slots();
    for (item, slot) in mapping {
        sc = add_item(sc, item, slots[slot.index()].center(), INK, 1.8);
    }
    sc
}

fn bottom_text(mut sc: Scene, line_a: &'static str, line_b: &'static str) -> Scene {
    let text_slots = bottom_text_slots();
    sc = sc.add(centered_text(&text_slots[0], line_a, 19.0, INK));
    sc.add(centered_text(&text_slots[1], line_b, 16.0, MUTED))
}

fn static_scene(
    title: &'static str,
    mapping: [(ItemId, SlotId); 2],
    active: Option<(SlotId, SlotId)>,
) -> Scene {
    let mut sc = background(scene());
    sc = add_slots(sc, active);
    sc = add_mapping(sc, mapping);
    bottom_text(
        sc,
        title,
        "slot positions are layout; A and B are concept items",
    )
}

fn swap_scene(events: &[SwapABEvent; 2], motion: SwapABMotion) -> Scene {
    let mut sc = background(scene());
    let slots = slots();
    sc = add_slots(sc, Some((SlotId::Left, SlotId::Right)));

    for event in events {
        let from = slots[event.from.index()].center().resolve(0.0);
        let to = slots[event.to.index()].center().resolve(0.0);
        let lane = match event.item {
            ItemId::A => 70.0,
            ItemId::B => -70.0,
        };
        sc = add_item(
            sc,
            event.item,
            motion.swap_path(from, to, lane),
            ACTIVE,
            motion.pulse(2.2, 4.0),
        );
    }

    bottom_text(
        sc,
        "Swap event: A moves slot 0 -> 1, B moves slot 1 -> 0",
        "the container never swaps; View maps stable item IDs to slot positions",
    )
}
