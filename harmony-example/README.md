# napi-ohos test example

This project is used for napi test.

## How to use
1. Setup environment with rust and harmony.
2. Setup `just`
3. Run build command with `just test`
4. Open DevEco Studio and run `All Tests in 'test(entry)'`
5. Use hdc to run all tests.
   ```bash
   hdc shell aa test -b com.ohos.napi -m entry_test -s unittest /ets/testrunner/OpenHarmonyTestRunner -s timeout 15000
   ```