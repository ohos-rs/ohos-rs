import "../runtime/console_shim";
import suite from "../src/compat-mode/Spawn.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Spawn.test.ts", suite, "/tmp");
