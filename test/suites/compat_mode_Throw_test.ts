import "../runtime/console_shim";
import suite from "../source/compat-mode/Throw.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Throw.test.ts", suite, "/tmp");
