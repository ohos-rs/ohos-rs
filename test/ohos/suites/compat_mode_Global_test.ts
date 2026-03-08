import "../runtime/console_shim";
import suite from "../src/compat-mode/Global.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Global.test.ts", suite, "/tmp");
