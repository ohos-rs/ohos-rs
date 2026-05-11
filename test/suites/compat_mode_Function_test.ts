import "../runtime/console_shim";
import suite from "../source/compat-mode/Function.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Function.test.ts", suite, "/tmp");
