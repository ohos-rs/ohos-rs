import "../runtime/console_shim";
import suite from "../src/compat-mode/String.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/String.test.ts", suite, "/tmp");
