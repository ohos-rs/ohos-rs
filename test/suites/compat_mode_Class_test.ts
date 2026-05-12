import "../runtime/console_shim";
import suite from "../source/compat-mode/Class.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Class.test.ts", suite, "/tmp");
