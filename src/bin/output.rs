//! Test application to list all available outputs.

use std::error::Error;

use sctk::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputInfo, OutputState},
    reexports::client::{protocol::wl_output, Connection, QueueHandle},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let registry_state = RegistryState::new(&conn, &qh);
    let output_delegate = OutputState::new();
    let mut list_outputs = ListOutputs {
        registry_state,
        output_state: output_delegate,
    };
    while !list_outputs.registry_state.ready() {
        event_queue.blocking_dispatch(&mut list_outputs)?;
    }
    event_queue.roundtrip(&mut list_outputs)?;
    for output in list_outputs.output_state.outputs() {
        print_output(
            &list_outputs
                .output_state
                .info(&output)
                .ok_or_else(|| "output has no info".to_owned())?,
        );
    }

    Ok(())
}
struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
}

impl OutputHandler for ListOutputs {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}
delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);
impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        OutputState,
    }
}
fn print_output(info: &OutputInfo) {
    println!("{}", info.model);

    if let Some(name) = info.name.as_ref() {
        println!("\tname: {}", name);
    }

    if let Some(description) = info.description.as_ref() {
        println!("\tdescription: {}", description);
    }

    println!("\tmake: {}", info.make);
    println!("\tx: {}, y: {}", info.location.0, info.location.1);
    println!("\tsubpixel: {:?}", info.subpixel);
    println!(
        "\tphysical_size: {}×{}mm",
        info.physical_size.0, info.physical_size.1
    );
    println!("\tmodes:");

    for mode in &info.modes {
        println!("\t\t{}", mode);
    }
}
