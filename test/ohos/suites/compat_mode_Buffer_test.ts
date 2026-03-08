import "../runtime/console_shim";
import suite from "../src/compat-mode/Buffer.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Buffer.test.ts", suite, "/tmp");
