import "../runtime/console_shim";
import suite from "../source/compat-mode/napi6/Bigint.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi6/Bigint.test.ts", suite, "/tmp");
