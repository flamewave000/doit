#include "doit.hpp"
#include <iostream>
#include <cstring>
#include <vector>
#include <algorithm>

#define __VAR(variable) {#variable, ::doit::to_string(variable)}
#define __SYSTEM(statement, vars) system(::doit::inject(statement, argc, argv, vars).c_str())
namespace script {
	double my_value = 42;
	// this is a comment and everything to the end of the line is ignored
	void build(int argc, const char *argv[]) {
		__SYSTEM_SH(R"__DOIT__(echo "hello $1, how are you? Here is my value: $(my_value)" | cat)__DOIT__", {__VAR(my_value)});
	}
	void clean(int argc, const char *argv[]) {
		__SYSTEM(R"__DOIT__(
echo "\$$1 \$$2 \$$3 \$$4 = $1 $2 $3 $4"
echo "     \$$(1:4) = $(1:4)"
echo "         \$$@ = $@"
echo "      \$$(3:) = $(3:@)"
echo "     \$$(3: ) = $(3: )"
echo "  \$$my_value = $my_value"
echo "\$$(my_value) = $(my_value)"
		)__DOIT__", {__VAR(my_value)});
	}
	void run(int argc, const char *argv[]) {
	}
}
#undef __VAR
#undef __VARS
#undef __SYSTEM

#define __HELP(target, help) {#target, ::doit::trim(help)}
typedef ::std::pair<::std::string, ::std::string> __target_help;
void print_help() {
	::std::string line;
	::std::stringstream is;
	printf("Usage: doit <target> [args...]\n");
	auto help_description = ::doit::trim(R"__DOIT__(Wazzup, here is my silly program
I hope you enjoy it)__DOIT__");
	if (help_description.size() > 0) {
		is = ::std::stringstream(help_description);
		while (::std::getline(is, line)) {
			printf("       %s\n", line.c_str());
		}
	}
	printf("\nTARGETS\n");
	::std::vector<__target_help> targets = {
		__HELP(build, R"__DOIT__(Build program
wazzup!!
Homies!!
)__DOIT__"),
		__HELP(clean, R"__DOIT__(Clean program)__DOIT__"),
		__HELP(run, R"__DOIT__(Run program)__DOIT__"),
	};
	int largest = 0;
	::std::sort(targets.begin(), targets.end(), [&largest](__target_help a, __target_help b) {
		largest = ::std::max(largest, ::std::max((int)a.first.size(), (int)b.first.size()));
		return a.first.compare(b.first);
	});
	for (auto target : targets) {
		is = ::std::stringstream(target.second);
		::std::getline(is, line);
		printf("  %*s  %s\n", largest, target.first.c_str(), line.c_str());
		while (::std::getline(is, line)) {
			printf("  %*s  %s\n", largest, "", line.c_str());
		}
	}
}
#undef __HELP

#define __MATCH(pattern) else if (!strcmp(argv[1], #pattern)) ::script::pattern(argc - 1, argv + 1)
int main(int argc, const char *argv[]) {
	if (argc < 2) {
		print_help();
		return EXIT_FAILURE;
	}
	if (!strcmp(argv[1], "--help"))
		print_help();
	__MATCH(build);
	__MATCH(clean);
	__MATCH(run);
	else {
		printf("Invalid target name: %s\nUsage: doit <target> [args...]\n", argv[1]);
		return EXIT_FAILURE;
	}
	return EXIT_SUCCESS;
}
#undef MATCH
