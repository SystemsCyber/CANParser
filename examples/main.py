import can_parser_python as can_parser

spec = ""
with open("/home/user_name/CANParser/J1939db_2020.json", "r") as f:
    spec = f.read()


specs = {
    can_parser.SPEC_TYPE_J1939: spec,
}

parser = can_parser.CANParserPython(can_parser.ERROR_WARN, r"^\((?P<timestamp>[0-9]+\.[0-9]+)\).*?(?P<id>[0-9A-F]{3,8})#(?P<data>[0-9A-F]+)", specs)

with open("/home/user_name/CANParser/highway2City.log", "r") as f:
    parser.parse_lines(f.readlines())

# Or you can use parse_file
# parser.parse_file("/home/user_name/CANParser/highway2City.log")