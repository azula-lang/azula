require "../ast/*"
require "../token"
require "../errors/*"

macro return_if_nil(val)
    if {{val}}.nil?
        return
    end
end

macro convert_and_check_nil(val)
    node = node.as?(AST::{{val}})
    return_if_nil node
end

module Azula
    module Types

        class Typechecker
            
            @variables : Hash(String, Type)
            @structs : Hash(String, Hash(String, Type))
            @function_return : Hash(String, Type)
            @function_args : Hash(String, Array(Type))

            getter errors

            def initialize
                @variables = {} of String=>Type
                @structs = {} of String=>Hash(String, Type)
                @function_return = {} of String=>Type
                @function_args = {} of String=>Array(Type)

                @function_return["cstring_conv"] = Type.new TypeEnum::CSTRING
                @function_args["cstring_conv"] = [] of (Type) << Type.new TypeEnum::STRING
            end

            def check(node : AST::Node) : Type?
                case node
                when .is_a?(AST::Program)
                    convert_and_check_nil Program
                    node.statements.not_nil!.each do |stmt|
                        val = self.check stmt
                        return_if_nil val
                        if val.main_type == TypeEnum::VOID
                            next
                        end
                        return val
                    end
                    return Type.new TypeEnum::VOID
                when .is_a?(AST::Block)
                    convert_and_check_nil Block
                    node.statements.each do |stmt|
                        val = self.check stmt
                        return_if_nil val
                        if val.main_type == TypeEnum::VOID
                            next
                        end
                        return val
                    end
                    return Type.new TypeEnum::VOID
                when .is_a?(AST::IntegerLiteral)
                    convert_and_check_nil IntegerLiteral
                    case node.size
                    when 8
                        return Type.new TypeEnum::INT8
                    when 16
                        return Type.new TypeEnum::INT16
                    when 32
                        return Type.new TypeEnum::INT
                    when 64
                        return Type.new TypeEnum::INT64
                    end
                    return Type.new TypeEnum::INT
                when .is_a?(AST::StringLiteral)
                    return Type.new TypeEnum::STRING
                when .is_a?(AST::FloatLiteral)
                    return Type.new TypeEnum::FLOAT
                when .is_a?(AST::BooleanLiteral)
                    return Type.new TypeEnum::BOOL
                when .is_a?(AST::Struct)
                    convert_and_check_nil Struct
                    fields = {} of String => Type
                    node.fields.each do |f|
                        fi = self.check f
                        return_if_nil fi
                        fields[f.ident] = fi
                    end
                    @structs[node.struct_name.ident] = fields
                    return Type.new TypeEnum::VOID
                when .is_a?(AST::StructInitialise)
                    convert_and_check_nil StructInitialise
                    name = node.struct_name.ident
                    str = @structs.fetch name, nil
                    if str.nil?
                        self.add_error "undefined struct #{name}", node.token
                        return
                    end
                    if node.values.size != str.values.size
                        self.add_error "incorrect number of attributes in struct, need #{str.values.size}, got #{node.values.size}", node.token
                        return
                    end
                    val_types = [] of Type
                    node.values.each do |val|
                        v = self.check val
                        return_if_nil v
                        val_types << v
                    end
                    str.values.size.times do |i|
                        if !compare_type str.values[i], val_types[i]
                            self.add_error "incorrect type in struct initialisation, cannot assign #{val_types[i].main_type} to type #{str.values[i].main_type}", node.token
                            return
                        end
                    end
                    return Type.type_from_string name
                when .is_a?(AST::ExpressionStatement)
                    convert_and_check_nil ExpressionStatement
                    self.check node.expression
                    return Type.new TypeEnum::VOID
                when .is_a?(AST::Identifier)
                    convert_and_check_nil Identifier
                    type = @variables.fetch node.ident, nil
                    if type.nil?
                        self.add_error "undefined variable #{node.ident}", node.token
                        return Type.new TypeEnum::VOID
                    end
                    return type
                when .is_a?(AST::TypedIdentifier)
                    convert_and_check_nil TypedIdentifier
                    return node.type
                when .is_a?(AST::Assign)
                    convert_and_check_nil Assign
                    val_types = [] of Type
                    node.values.each do |val|
                        v = self.check val
                        return_if_nil v
                        val_types << v
                    end
                    ident_type = self.check(node.idents[0]).not_nil!
                    val_types.size.times do |i|
                        if !compare_type val_types[i], ident_type
                            self.add_error "incorrect value in assign statement, trying to assign #{val_types[i].main_type} to variable of type #{ident_type.main_type}", node.token
                            return Type.new TypeEnum::VOID
                        end
                    end
                    node.idents.each do |val|
                        v = val.as?(AST::TypedIdentifier)
                        if v.nil?
                            v = val.as?(AST::Identifier)
                            if v.nil?
                                self.add_error "not an identifier", node.token
                                return
                            end
                            @variables[v.not_nil!.ident] = ident_type
                            next
                        end
                        @variables[v.ident] = ident_type
                    end
                    return Type.new TypeEnum::VOID
                when .is_a?(AST::Return)
                    convert_and_check_nil Return
                    return self.check node.values[0]
                when .is_a?(AST::Function)
                    convert_and_check_nil Function

                    old_vars = @variables
                    params = [] of Type
                    node.parameters.each do |param|
                        type = self.check param
                        return_if_nil type
                        if !self.is_valid_type type
                            self.add_error "undefined struct #{type}", node.token
                            return
                        end
                        params << type
                        @variables[param.ident] = type
                    end
                    @function_args[node.function_name.ident] = params

                    @function_return[node.function_name.ident] = node.return_type

                    body_return = self.check node.body
                    return_if_nil body_return

                    if !compare_type body_return, node.return_type
                        self.add_error "cannot return #{body_return.main_type}, requires #{node.return_type.main_type}", node.token
                        return
                    end

                    @variables = old_vars
                    return Type.new TypeEnum::VOID
                when .is_a?(AST::ExternFunction)
                    convert_and_check_nil ExternFunction

                    old_vars = @variables
                    params = [] of Type
                    node.parameters.each do |param|
                        type = self.check param
                        return_if_nil type
                        if !self.is_valid_type type
                            self.add_error "undefined struct #{type.main_type}", node.token
                            return
                        end
                        params << type
                        @variables[param.ident] = type
                    end
                    @function_args[node.function_name.ident] = params

                    @function_return[node.function_name.ident] = node.return_type

                    return Type.new TypeEnum::VOID
                when .is_a?(AST::FunctionCall)
                    convert_and_check_nil FunctionCall
                    name = node.function_name.ident

                    args = [] of Type
                    node.arguments.each do |arg|
                        t = self.check arg
                        return_if_nil t
                        args << t
                    end

                    if name == "println"
                        return Type.new TypeEnum::VOID
                    end
                    str = @function_return.fetch name, nil
                    if str.nil?
                        self.add_error "undefined function #{name}", node.token
                        return
                    end

                    arg_types = @function_args.fetch name, nil
                    if arg_types.nil?
                        self.add_error "undefined function #{name}", node.token
                        return
                    end

                    if args.size != arg_types.size
                        self.add_error "incorrect number of arguments, got #{args.size}, want #{arg_types.size}", node.token
                        return
                    end

                    args.size.times do |i|
                        if !compare_type args[i], arg_types[i]
                            self.add_error "cannot assign #{args[i].main_type} to argument of type #{arg_types[i]}", node.token
                            return
                        end
                    end

                    return str
                when .is_a?(AST::Infix)
                    convert_and_check_nil Infix
                    case node.operator
                    when "*", "+", "-", "/", "%"
                        return self.check node.left
                    when "==", "!=", "or", "and", "<=", ">="
                        return Type.new TypeEnum::BOOL
                    end
                when .is_a?(AST::ArrayExp)
                    convert_and_check_nil ArrayExp
                    arrayType : Types::Type? = nil
                    node.values.each do |v|
                        t = self.check v
                        return_if_nil t
                        if arrayType == nil
                            arrayType = t
                            node.type.secondary_type = t
                            next
                        end
                        if t.not_nil!.main_type != arrayType.not_nil!.main_type
                            self.add_error "cannot have array of multiple types", node.token
                            return
                        end
                    end
                    return node.type
                when .is_a?(AST::ArrayAccess)
                    convert_and_check_nil ArrayAccess
                    v = self.check(node.array)
                    return_if_nil v

                    if v.main_type != Types::TypeEnum::ARRAY
                        self.add_error "can't index non-array", node.token
                        return
                    end

                    index_type = self.check(node.index)
                    return_if_nil index_type
                    if !index_type.is_int
                        self.add_error "can't use non-int as index", node.token
                        return
                    end

                    return v.not_nil!.secondary_type
                else
                    return Type.new TypeEnum::VOID
                end
            end

            def add_error(s : String, token : Token)
                ErrorManager.add_error Error.new "#{s}", token.file, token.linenumber, token.charnumber
            end

            def is_valid_type(s : Type)
                t = s.main_type.as?(TypeEnum)
                if t.nil?
                    g = @structs.fetch s.main_type, nil
                    return !g.nil?
                end
                return true
            end

            def compare_type(t : Type, t1 : Type) : Bool
                if t.main_type == t1.main_type
                    return true
                end

                if t.main_type == TypeEnum::INT || t1.main_type == TypeEnum::INT
                    return t.is_int && t1.is_int
                end

                return false
            end

        end
    end
end