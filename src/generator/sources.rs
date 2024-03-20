pub const DOIT_HEADER: &str = r#"#include <unordered_map>
#include <sstream>
namespace doit {
	typedef ::std::unordered_map<::std::string, ::std::string> args_map;
	inline ::std::string to_string(::std::string __val) { return __val; }
	::std::string to_string(double __val) {
		auto result = ::std::to_string(__val);
		size_t end = result.find_last_not_of('0');
		if (result[end] != '.') end++;
		result.erase(end, ::std::string::npos);
		return result;
	}
	inline ::std::string trim(::std::string text) {
		auto first = text.find_first_not_of(" \t\n\r\f\v");
		auto last = text.find_last_not_of(" \t\n\r\f\v");
		if (first == ::std::string::npos || last == ::std::string::npos) return text;
		return text.substr(first, (last - first) + 1);
	}
	::std::string inject(::std::string fmt, args_map vars) {
		::std::ostringstream os;
		::std::string token = "";
		bool parsing = false;
		bool bracketed = false;
		for (size_t c = 0, size = fmt.size(); c < size; c++) {
			if (parsing) {
				if (bracketed) {
					if (fmt[c] == ')') {
						bracketed = false;
						parsing = false;
						os << vars[token];
						token = "";
					}
					else
						token += fmt[c];
				} else {
					if (!isalnum(fmt[c]) && fmt[c] != '_') {
						parsing = false;
						os << vars[token] << fmt[c];
						token = "";
					} else
						token += fmt[c];
				}
			} else {
				if (fmt[c] != '$')
					os << fmt[c];
				else {
					if (c + 1 >= size) {
						printf("Unexpected '$' at end of line");
						exit(EXIT_FAILURE);
					}
					else if (fmt[c + 1] == '$') {
						c++;
						os << '$';
					}
					else {
						parsing = true;
						if (fmt[c + 1] == '(') {
							bracketed = true;
							c++;
						}
					}
				}
			}
		}
		if (token.size() > 0)
			os << vars[token];
		return os.str();
	}
	inline args_map concat(args_map &target, const args_map &elements) {
		args_map result;
		result.insert(target.begin(), target.end());
		result.insert(elements.begin(), elements.end());
		return result;
	}
}
"#;

pub const SOURCE_FILE: &str = r#"#include <iostream>
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
			printf("%s\n", line.c_str());
		}
		printf("\n");
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
"#;