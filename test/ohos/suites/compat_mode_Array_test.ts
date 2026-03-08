import "../runtime/console_shim";
import suite from "../src/compat-mode/Array.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Array.test.ts", suite, "/tmp");
