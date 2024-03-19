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
    inline args_map &concat(args_map &target, const args_map &elements) {
        target.insert(elements.begin(), elements.end());
        return target;
    }
}

#include <iostream>
#include <cstring>
#include <cstdlib>

#define __VAR(variable) {#variable, ::doit::to_string(variable)}
#define __VARS(...) ::doit::concat(__args, {__VA_ARGS__})
#define __SYSTEM(statement, vars) system(::doit::inject(statement, vars).c_str())
namespace script {
    double my_value = 42;
    double my_value2 = 21+3+my_value;
    ::std::string myval3 = "hello world";
    // this is a comment and everything to the end of the line is ignored
    __SYSTEM(R"__DOIT__(echo "hello world" | cat)__DOIT__", __VARS(__VAR(my_value),__VAR(my_value2),__VAR(myval3)));
    void build(::doit::args_map __args) {
        double my_value2 = 21+3+my_value;
        ::std::string my_str = "Hello World";
        __SYSTEM(R"__DOIT__(echo "hello world" | cat)__DOIT__", __VARS(__VAR(my_value),__VAR(my_value2),__VAR(myval3),__VAR(my_value2),__VAR(my_str)));
        __SYSTEM(R"__DOIT__(echo "$1" | cat)__DOIT__", __VARS(__VAR(my_value),__VAR(my_value2),__VAR(myval3),__VAR(my_value2),__VAR(my_str)));
        __SYSTEM(R"__DOIT__(echo $(my_value))__DOIT__", __VARS(__VAR(my_value),__VAR(my_value2),__VAR(myval3),__VAR(my_value2),__VAR(my_str)));
    }
    void clean(::doit::args_map __args) {
        __SYSTEM(R"__DOIT__(echo "hello world" | cat)__DOIT__", __VARS(__VAR(my_value),__VAR(my_value2),__VAR(myval3)));
    }
    void run(::doit::args_map __args) {
    }
}
#undef __VAR
#undef __VARS
#undef __SYSTEM

#define __MATCH(pattern) else if (!strcmp(argv[1], #pattern)) ::script::pattern(args)
int main(int argc, const char *argv[]) {
    if (argc < 2) {
        ::std::cout << "Targets: ["
            << "build" ", "
            << "clean" ", "
            << "run"
            << "]\n" << ::std::flush;
        return EXIT_FAILURE;
    }
    ::doit::args_map args;
    for (size_t i = 1; i < argc; i++)
        args[::std::to_string(i-1)] = argv[i];
    if (!strcmp(argv[1], "--help"))
        printf("Usage: doit <target> [args...]\n");
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