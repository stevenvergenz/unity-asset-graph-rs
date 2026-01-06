using System;
using My.DifferentNamespace;

namespace My.Namespace {
    public class MyClass {
        public delegate void MyDelegate(int x);

        internal class UnderClass { }

        private static My.OtherNamespace.LocalizedString locstringNormal = LocStringCache.Get("NormalKey");

        private static LocalizedString locstringPrefixed = LocStringCache.Get(
            key: "PrefixedKey",
            formatArgs: "Some other text");

        private const string someKey = "RefKey";

        private static LocalizedString locstringRef = LocStringCache.Get(someKey);

        private static LocalizedString locstringRefPrefix = LocStringCache.Get(key: someKey);

        public int MyProperty { get; set; }

        public static void MyMethod()
        {
            LocStringCache.Deep.FakeProp = someKey.Length;
        }
    }

    struct MyStruct {
        public int X;
        public int Y;
    }

    enum MyEnum {
        First,
        Second,
        Third
    }

    interface IMyInterface {
        void DoSomething();
    }

    namespace InnerNamespace {
        class InnerClass { }
    }
}