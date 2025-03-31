//use instruction_statistic::InstructionStatistic;

use specs::slice_backend::SliceBackend;
use specs::Tables;

mod helper;
// mod instruction_statistic;
#[allow(dead_code)]
pub trait Profiler {
    fn profile_tables(&self);
}

impl<B: SliceBackend> Profiler for Tables<B> {
    fn profile_tables(&self) {
        //self.profile_instruction();
    }
}
