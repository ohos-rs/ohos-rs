import "../runtime/console_shim";
import suite from "../source/compat-mode/serde/SerdeJson.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/serde/SerdeJson.test.ts", suite, "/tmp");
