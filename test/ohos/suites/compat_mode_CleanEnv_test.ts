import "../runtime/console_shim";
import suite from "../src/compat-mode/CleanEnv.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/CleanEnv.test.ts", suite, "/tmp");
