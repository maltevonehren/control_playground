use std::rc::Rc;

use engine::dynamic_system::{CompoundSystem, CompoundSystemComponent};
use leptos::*;

#[component]
pub fn SVGSystemDiagram(sys: Rc<CompoundSystem>) -> impl IntoView {
    view! {
        <div style:overflow="hidden">
        <svg width="100%" height="100%" style:stroke-width="2px" >
            { sys.components
                .iter().enumerate()
                .map(|(i, block)| make_state_space_block(i, block))
                .collect_view() }
        </svg>
        </div>
    }
}

fn make_state_space_block(pos: usize, block: &CompoundSystemComponent) -> impl IntoView {
    view! {
        <rect y="10" x={10 + 120 * pos} width="100" height="80" rx="15"
            style:stroke="black" style:fill="white" />
        <text y="30" x={30 + 120 * pos} style:font-size="7pt"> { format!("{}", block.block) } </text>
        <text y="110" x={30 + 120 * pos}> { block.name.clone() } </text>
    }
}
