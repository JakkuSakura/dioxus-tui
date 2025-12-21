#![cfg(feature = "blitz")]

use std::any::Any;

use dioxus_core::ComponentFunction;

use crate::RawVirtualDom;

pub(crate) fn launch_blitz_gui<P, F>(raw: RawVirtualDom<P, F>)
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let (app, props, contexts): (
        F,
        P,
        Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    ) = raw.into_parts();

    dioxus_native::launch_cfg_with_props(app, props, contexts, Vec::new());
}
