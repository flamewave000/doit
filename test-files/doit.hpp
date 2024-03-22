#include <unordered_map>
#include <sstream>
#include <regex>
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
					if (!isalnum(fmt[c]) && fmt[c] != '_' && fmt[c] != '@') {
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
