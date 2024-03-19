#include <iostream>
#include <cstring>
#include <vector>
#include <algorithm>

#define __VAR(variable) {#variable, ::doit::to_string(variable)}
#define __VARS(...) ::doit::concat(__args, {__VA_ARGS__})
#define __SYSTEM(statement, vars) system(::doit::inject(statement, vars).c_str())
namespace script {
{{{TARGET_DEFINITIONS}}}
}
#undef __VAR
#undef __VARS
#undef __SYSTEM

#define __HELP(target, help) {#target, ::doit::trim(help)}
typedef ::std::pair<::std::string, ::std::string> __target_help;
void print_help() {
	::std::string line;
	::std::stringstream is;
	auto help_description = ::doit::trim(R"__DOIT__({{{ROOT_HELP}}})__DOIT__");
	if (help_description.size() > 0) {
		is = ::std::stringstream(help_description);
		while (::std::getline(is, line)) {
			printf("       %s\n", line.c_str());
		}
	}
	printf("Usage: doit <target> [args...]\n");
	printf("\nTARGETS\n");
	::std::vector<__target_help> targets = {{{{TARGET_HELPS}}}
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

#define __MATCH(pattern) else if (!strcmp(argv[1], #pattern)) ::script::pattern(args)
int main(int argc, const char *argv[]) {
	if (argc < 2) {
		print_help();
		return EXIT_FAILURE;
	}
	::doit::args_map args;
	for (size_t i = 1; i < argc; i++)
		args[::std::to_string(i-1)] = argv[i];
	if (!strcmp(argv[1], "--help"))
		print_help();{{{TARGET_MATCHES}}}
	else {
		printf("Invalid target name: %s\nUsage: doit <target> [args...]\n", argv[1]);
		return EXIT_FAILURE;
	}
	return EXIT_SUCCESS;
}
#undef MATCH
