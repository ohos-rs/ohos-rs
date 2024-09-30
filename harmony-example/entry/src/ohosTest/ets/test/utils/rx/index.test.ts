// 模拟 Subject 类
export class Subject {
  observers: any;

  constructor() {
    this.observers = [];
  }

  next() {
    this.observers.forEach((observer) => observer());
  }

  pipe(operator) {
    return operator(this);
  }

  subscribe(observer) {
    this.observers.push(observer);
    return {
      unsubscribe: () => {
        this.observers = this.observers.filter((obs) => obs !== observer);
      }
    };
  }
}

// 模拟 take 操作符
export function take(count) {
  return function (subject) {
    return {
      subscribe(observer) {
        let taken = 0;
        const subscription = subject.subscribe(() => {
          taken++;
          observer();
          if (taken >= count) {
            subscription.unsubscribe();
          }
        });
        return subscription;
      }
    };
  };
}
