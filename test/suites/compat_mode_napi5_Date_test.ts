import "../runtime/console_shim";
import suite from "../source/compat-mode/napi5/Date.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi5/Date.test.ts", suite, "/tmp");
