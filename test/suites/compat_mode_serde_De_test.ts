import "../runtime/console_shim";
import suite from "../source/compat-mode/serde/De.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/serde/De.test.ts", suite, "/tmp");
