pub const DOIT_HEADER: &str = r#"#include <unordered_map>
#include <sstream>
#include <regex>
namespace doit {
	int EXIT_CODE = 0;
	void exit(int override = -1) { ::exit(override < 0 ? EXIT_CODE : override); }
	void yield() { if (EXIT_CODE > 0) exit(EXIT_CODE); }
	typedef ::std::unordered_map<::std::string, ::std::string> args_map;
	inline ::std::string to_string(::std::string __val) { return __val; }
	struct __target_help_args {
		bool required;
		::std::string arg_name;
		::std::string arg_help;
	};
	struct __target_help {
		::std::string target_name;
		::std::string target_help;
		::std::vector<__target_help_args> target_args;
	};
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
	void print_tabbed_text(const ::uint16_t tabwidth, const bool clip_start, const ::std::string &text) {
		::std::string line;
		::std::stringstream is = ::std::stringstream(text);
		if (clip_start) {
			::std::getline(is, line);
			printf("  %s\n", ::doit::trim(line).c_str());
		}
		while (::std::getline(is, line)) {
			printf("  %*s  %s\n", tabwidth, "", ::doit::trim(line).c_str());
		}
	}
	::std::string inject(::std::string fmt, int argc, const char *argv[], args_map vars) {
		::std::ostringstream os;
		::std::string token = "";
		bool parsing = false;
		bool bracketed = false;
		auto get = [&vars, argc, argv](const ::std::string &key) -> ::std::string {
			// if we are matching a single CLI arg
			if (::std::all_of(key.begin(), key.end(), ::isdigit)) {
				int64_t idx = atoll(key.c_str());
				if (idx < 0 || idx >= argc) return "";
				return argv[idx];
			}
			// If we are matching ALL available CLI args
			if (key.size() == 1 && key[0] == '@') {
				::std::ostringstream oss;
				for (int c = 1; c < argc; c++) {
					oss << argv[c];
					if (c < argc - 1) oss << ' ';
				}
				return oss.str();
			}
			// If we are matching the number of CLI arguments
			if (key.size() == 1 && key[0] == '#') {
				return (::std::ostringstream() << (argc - 1)).str();
			}
			::std::regex pattern("(\\d+):(\\d+)?");
			::std::smatch matches;
			// If we are matching a range af CLI args
			if (::std::regex_search(key, matches, pattern)) {
				int first = atoll(matches[1].str().c_str());
				auto match2 = matches[2].str();
				int last = match2.size() == 0 ? argc - 1 : atoll(matches[2].str().c_str());
				if (first < last && first >= 0) {
					::std::ostringstream oss;
					for (int c = first, size = ::std::min(last + 1, argc); c < size; c++) {
						oss << argv[c];
						if (c < size - 1) oss << ' ';
					}
					return oss.str();
				}
			}
			return vars[key];
		};
		for (size_t c = 0, size = fmt.size(); c < size; c++) {
			if (parsing) {
				if (bracketed) {
					if (fmt[c] == ')') {
						bracketed = false;
						parsing = false;
						os << get(token);
						token = "";
					}
					else
						token += fmt[c];
				} else {
					if (!isalnum(fmt[c]) && fmt[c] != '_' && fmt[c] != '@' && fmt[c] != '#') {
						parsing = false;
						os << get(token) << fmt[c];
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
		if (token.size() > 0) {
			os << get(token);
		}
		return os.str();
	}
}
"#;

pub const SOURCE_FILE: &str = r#"#include <iostream>
#include <cstring>
#include <vector>
#include <algorithm>
#include <sys/wait.h>

#define __VAR(variable) {#variable, ::doit::to_string(variable)}
#define __SYSTEM_SH(statement, vars) ::doit::EXIT_CODE = WEXITSTATUS(system(::doit::inject(statement, argc, argv, vars).c_str()))
#define __SYSTEM_PY(statement, vars) ::doit::EXIT_CODE = WEXITSTATUS(system(("cat <<__EOF__ | python3\n" + ::doit::inject(statement, argc, argv, vars) + "\n__EOF__\n").c_str()))
namespace script {
{{{TARGET_DEFINITIONS}}}
}
#undef __VAR
#undef __VARS
#undef __SYSTEM

#define __ARG(req, arg, help) ::doit::__target_help_args{req, arg, ::doit::trim(help)}
#define __HELP(target, help, ...) {#target, ::doit::trim(help), {__VA_ARGS__}}
void print_help() {
	::std::string line;
	::std::stringstream is;
	printf("\e[32mUsage: \e[33mdoit \e[34m<target> \e[90m[args...]\e[0m\n");
	auto help_description = ::doit::trim(R"__DOIT__({{{ROOT_HELP}}})__DOIT__");
	if (help_description.size() > 0) {
		is = ::std::stringstream(help_description);
		while (::std::getline(is, line)) {
			printf("       %s\n", line.c_str());
		}
	}
	printf("\n\e[32mTARGETS\e[0m\n");
	::std::vector<::doit::__target_help> targets = {{{{TARGET_HELPS}}}
	};
	int largest = 0;
	::std::sort(targets.begin(), targets.end(), [&largest](const ::doit::__target_help &a, const ::doit::__target_help &b) {
		largest = ::std::max(largest, ::std::max((int)a.target_name.size(), (int)b.target_name.size()));
		return a.target_name < b.target_name;
	});
	for (auto target : targets) {
		printf("\e[34m  %*s\e[0m", largest, target.target_name.c_str());
		if (target.target_args.size() == 0) {
			::doit::print_tabbed_text(largest, true, target.target_help.c_str());
			continue;
		}
		int largest_arg = 0;
		for (auto arg : target.target_args) {
			if (arg.required)
				printf("\e[90m <%s>\e[0m", arg.arg_name.c_str());
			else
				printf("\e[90m [%s]\e[0m", arg.arg_name.c_str());
			largest_arg = ::std::max(largest_arg, (int)arg.arg_name.size());
		}
		::std::cout << ::std::endl;
		::doit::print_tabbed_text(largest, false, target.target_help.c_str());
		for (auto arg : target.target_args) {
			printf("\e[90m%*s\e[0m", largest + largest_arg + 4, arg.arg_name.c_str());
			::doit::print_tabbed_text(largest + largest_arg + 2, true, arg.arg_help.c_str());
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
		print_help();{{{TARGET_MATCHES}}}
	else {
		printf("\e[91mInvalid target name: \e[33m%s\e[0m\n\e[32mUsage: \e[34mdoit <target> [args...]\e[0m\n", argv[1]);
		return EXIT_FAILURE;
	}
	return ::doit::EXIT_CODE;
}
#undef MATCH
"#;