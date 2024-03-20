#include <unordered_map>
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
