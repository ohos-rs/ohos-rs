import "../runtime/console_shim";
import suite from "../src/compat-mode/CreateExternal.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/CreateExternal.test.ts", suite, "/tmp");
