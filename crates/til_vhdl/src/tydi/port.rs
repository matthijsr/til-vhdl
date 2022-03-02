use til_query::common::physical::signal_list::SignalList;
use tydi_vhdl::port::Port;

pub struct TydiPort<T> {
    signal_list: SignalList<T>,
    
}