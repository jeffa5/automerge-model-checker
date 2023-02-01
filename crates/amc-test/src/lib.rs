use amc::model::{ModelBuilder, ModelOpts};
use amc::report::TestReporter;
use expect_test::Expect;
use stateright::Checker;
use stateright::Model;

/// Run the model checker using BFS and assert against the expected output of the test reporter.
pub fn check_bfs<O>(model_opts: ModelOpts, app_opts: O, expected: Expect)
where
    O: ModelBuilder,
    O::History: Sync + Send + 'static,
    O::Config: Sync + Send,
{
    let model = model_opts.to_model(&app_opts);
    let mut reporter = TestReporter::default();
    model.checker().spawn_bfs().join_and_report(&mut reporter);
    expected.assert_eq(&reporter.data);
}
