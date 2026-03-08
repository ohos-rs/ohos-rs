import "../runtime/console_shim";
import suite from "../src/compat-mode/Either.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/Either.test.ts", suite, "/tmp");
