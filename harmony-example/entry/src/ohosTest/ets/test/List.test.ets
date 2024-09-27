import abilityTest from './Ability.test';
import errorMessage from './napi/ErrorMessage.test';

export default function testsuite() {
  abilityTest();
  errorMessage();
}