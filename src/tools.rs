use crate::state::CargoTools;
mod cargo_utils;
mcplease::tools!(
    CargoTools,
    (CargoCheck, cargo_check, "cargo_check"),
    (CargoClippy, cargo_clippy, "cargo_clippy"),
    (CargoTest, cargo_test, "cargo_test"),
    (CargoFmtCheck, cargo_fmt_check, "cargo_fmt_check"),
    (CargoBuild, cargo_build, "cargo_build"),
    (CargoBench, cargo_bench, "cargo_bench"),
    (CargoAdd, cargo_add, "cargo_add"),
    (CargoRemove, cargo_remove, "cargo_remove"),
    (CargoUpdate, cargo_update, "cargo_update"),
    (CargoClean, cargo_clean, "cargo_clean"),
    (
        SetWorkingDirectory,
        set_working_directory,
        "set_working_directory"
    ),
    (CargoRun, cargo_run, "cargo_run")
);
