import "../runtime/console_shim";
import suite from "../source/compat-mode/napi4/TSFNError.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi4/TSFNError.test.ts", suite, "/tmp");
