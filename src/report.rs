use stateright::Model;

#[derive(Debug, Default)]
pub struct Reporter {
    last_total: usize,
    last_unique: usize,
}

impl<M> stateright::report::Reporter<M> for Reporter
where
    M: Model,
{
    fn report_checking(&mut self, data: stateright::report::ReportData) {
        let new_total = data.total_states - self.last_total;
        let total_rate = (data.total_states as f64 / data.duration.as_secs_f64()).round();
        let new_unique = data.unique_states - self.last_unique;
        let unique_rate = (data.unique_states as f64 / data.duration.as_secs_f64()).round();
        let status = if data.done { "Done    " } else { "Checking" };
        println!(
            "{} states={: >8} (+{: <8} {: >8.0}/s), unique={: >8} (+{: <8} {: >8}/s), duration={:?}",
            status,
            data.total_states,
            new_total,
            total_rate,
            data.unique_states,
            new_unique,
            unique_rate,
            data.duration
        );

        self.last_total = data.total_states;
        self.last_unique = data.unique_states;
    }

    fn report_discoveries(
        &mut self,
        discoveries: std::collections::HashMap<
            &'static str,
            stateright::report::ReportDiscovery<M>,
        >,
    ) where
        <M as Model>::Action: std::fmt::Debug,
        <M as Model>::State: std::fmt::Debug,
    {
        for (name, discovery) in discoveries {
            let _ = print!(
                "Discovered \"{}\" {} {}",
                name, discovery.classification, discovery.path,
            );
        }
    }
}
