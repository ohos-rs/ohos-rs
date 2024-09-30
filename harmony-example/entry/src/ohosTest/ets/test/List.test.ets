import abilityTest from './Ability.test';
import errorMessage from './napi/ErrorMessage.test';
import valueTest from './napi/value.test';

export default function testsuite() {
  abilityTest();
  errorMessage();
  valueTest();
}