import "../runtime/console_shim";
import suite from "../source/compat-mode/Env.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Env.test.ts", suite, "/tmp");
