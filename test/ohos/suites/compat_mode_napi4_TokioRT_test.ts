import "../runtime/console_shim";
import suite from "../src/compat-mode/napi4/TokioRT.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi4/TokioRT.test.ts", suite, "/tmp");
