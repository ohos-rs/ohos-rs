import "../runtime/console_shim";
import suite from "../src/compat-mode/Object.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Object.test.ts", suite, "/tmp");
