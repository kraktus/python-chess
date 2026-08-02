import inspect
import time
import sys
import os
import traceback

sys.path.insert(0, os.path.abspath("."))
import benchmarks


def run_benchmarks():
    # Iterate over all members of the benchmarks module
    errors = 0
    for name, obj in inspect.getmembers(benchmarks, inspect.isclass):
        if obj.__module__ == "benchmarks" or name.endswith("Suite"):
            print(f"Running suite: {name}")

            # Find all time_* methods
            time_methods = [
                m_name
                for m_name, m in inspect.getmembers(obj, callable)
                if m_name.startswith("time_")
            ]

            if not time_methods:
                continue

            instance = obj()

            for method_name in time_methods:
                # Call setup before each method for isolation
                if hasattr(instance, "setup"):
                    instance.setup()

                method = getattr(instance, method_name)
                start_time = time.time()
                try:
                    method()
                except Exception as e:
                    errors += 1
                    print(f"  Error in {method_name}: {e}")
                    traceback.print_exc()
                    print()
                end_time = time.time()

                print(f"  {method_name}: {end_time - start_time:.6f}s")

    print(f"Tests all run, with {errors} errors")


if __name__ == "__main__":
    run_benchmarks()
