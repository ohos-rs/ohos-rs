import "../runtime/console_shim";
import suite from "../source/compat-mode/serde/Ser.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/serde/Ser.test.ts", suite, "/tmp");
