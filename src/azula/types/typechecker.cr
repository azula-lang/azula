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
            
            @variables : Hash(String, (Type | String))
            @structs : Hash(String, Hash(String, (Type | String)))
            @function_returns : Hash(String, Array(Type | String))
            @function_args : Hash(String, Array(Type | String))

            getter errors

            def initialize
                @variables = {} of String=>(Type | String)
                @structs = {} of String=>Hash(String, (Type | String))
                @function_returns = {} of String=>Array(Type | String)
                @function_args = {} of String=>Array(Type | String)
            end

            def check(node : AST::Node) Array(Type | String)?
                case node
                when .is_a?(AST::Program)
                    convert_and_check_nil Program
                    node.statements.not_nil!.each do |stmt|
                        val = self.check stmt
                        return_if_nil val
                        if val.size == 0 || val[0] == Type::VOID
                            next
                        end
                        return val
                    end
                    return [Type::VOID]
                when .is_a?(AST::Block)
                    convert_and_check_nil Block
                    node.statements.each do |stmt|
                        val = self.check stmt
                        return_if_nil val
                        if val.size == 0 || val[0] == Type::VOID
                            next
                        end
                        return val
                    end
                    return [Type::VOID]
                when .is_a?(AST::IntegerLiteral)
                    convert_and_check_nil IntegerLiteral
                    case node.size
                    when 8
                        return [Type::INT8]
                    when 16
                        return [Type::INT16]
                    when 32
                        return [Type::INT]
                    when 64
                        return [Type::INT64]
                    end
                    return [Type::INT]
                when .is_a?(AST::StringLiteral)
                    return [Type::STRING]
                when .is_a?(AST::FloatLiteral)
                    return [Type::FLOAT]
                when .is_a?(AST::BooleanLiteral)
                    return [Type::BOOL]
                when .is_a?(AST::Struct)
                    convert_and_check_nil Struct
                    fields = {} of String => (Type | String)
                    node.fields.each do |f|
                        fi = self.check f
                        return_if_nil fi
                        fields[f.ident] = fi[0]
                    end
                    @structs[node.struct_name.ident] = fields
                    return [Type::VOID]
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
                    val_types = [] of (Type | String)
                    node.values.each do |val|
                        v = self.check val
                        return_if_nil v
                        val_types << v[0]
                    end
                    str.values.size.times do |i|
                        if str.values[i] != val_types[i]
                            self.add_error "incorrect type in struct initialisation, cannot assign #{val_types[i]} to type #{str.values[i]}", node.token
                            return
                        end
                    end
                    return [name]
                when .is_a?(AST::ExpressionStatement)
                    convert_and_check_nil ExpressionStatement
                    self.check node.expression
                    return [Type::VOID]
                when .is_a?(AST::Identifier)
                    convert_and_check_nil Identifier
                    type = @variables.fetch node.ident, nil
                    if type.nil?
                        self.add_error "undefined variable #{node.ident}", node.token
                        return [Type::VOID]
                    end
                    return [type]
                when .is_a?(AST::TypedIdentifier)
                    convert_and_check_nil TypedIdentifier
                    return [node.type]
                when .is_a?(AST::Assign)
                    convert_and_check_nil Assign
                    val_types = [] of (Type | String)
                    node.values.each do |val|
                        v = self.check val
                        return_if_nil v
                        val_types = val_types + v
                    end
                    ident_type = self.check(node.idents[0]).not_nil![0]
                    val_types.size.times do |i|
                        if !compare_type val_types[i], ident_type
                            self.add_error "incorrect value in assign statement, trying to assign #{val_types[i]} to variable of type #{ident_type}", node.token
                            return [Type::VOID]
                        end
                    end
                    node.idents.each do |val|
                        v = val.as?(AST::TypedIdentifier)
                        if v.nil?
                            v = val.as?(AST::Identifier)
                            if v.nil?
                                self.add_error "not an identifier", node.token
                            end
                            @variables[v.not_nil!.ident] = ident_type
                            next
                        end
                        @variables[v.ident] = ident_type
                    end
                    return [Type::VOID]
                when .is_a?(AST::Return)
                    convert_and_check_nil Return
                    vals = [] of (Type | String)
                    node.values.each do |val|
                        t = self.check val
                        return_if_nil t
                        vals << t[0]
                    end
                    return vals
                when .is_a?(AST::Function)
                    convert_and_check_nil Function

                    old_vars = @variables
                    params = [] of (Type | String)
                    node.parameters.each do |param|
                        type = self.check param
                        return_if_nil type
                        if !self.is_valid_type type[0]
                            self.add_error "undefined struct #{type[0]}", node.token
                            return
                        end
                        params << type[0]
                        @variables[param.ident] = type[0]
                    end
                    @function_args[node.function_name.ident] = params

                    @function_returns[node.function_name.ident] = node.return_types

                    body_return = self.check node.body
                    return_if_nil body_return

                    if body_return.size != node.return_types.size
                        self.add_error "incorrect number of return values, got #{body_return.size}, want #{node.return_types.size}", node.token
                        return
                    end

                    body_return.size.times do |i|
                        if !compare_type body_return[i], node.return_types[i]
                            self.add_error "cannot return #{body_return[i]}, requires #{node.return_types[i]}", node.token
                            return
                        end
                    end

                    @variables = old_vars
                    return [Type::VOID]
                when .is_a?(AST::ExternFunction)
                    convert_and_check_nil ExternFunction

                    old_vars = @variables
                    params = [] of (Type | String)
                    node.parameters.each do |param|
                        type = self.check param
                        return_if_nil type
                        if !self.is_valid_type type[0]
                            self.add_error "undefined struct #{type[0]}", node.token
                            return
                        end
                        params << type[0]
                        @variables[param.ident] = type[0]
                    end
                    @function_args[node.function_name.ident] = params

                    @function_returns[node.function_name.ident] = node.return_types

                    return [Type::VOID]
                when .is_a?(AST::FunctionCall)
                    convert_and_check_nil FunctionCall
                    name = node.function_name.ident
                    if name == "println"
                        return [Type::VOID]
                    end
                    str = @function_returns.fetch name, nil
                    if str.nil?
                        self.add_error "undefined function #{name}", node.token
                        return
                    end

                    args = [] of (Type | String)
                    node.arguments.each do |arg|
                        t = self.check arg
                        return_if_nil t
                        args << t[0]
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
                        if args[i] != arg_types[i]
                            self.add_error "cannot assign #{args[i]} to argument of type #{arg_types[i]}", node.token
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
                        return [Type::BOOL]
                    end
                else
                    return [Type::VOID]
                end
            end

            def add_error(s : String, token : Token)
                ErrorManager.add_error Error.new "#{s}", token.file, token.linenumber, token.charnumber
            end

            def is_valid_type(s : (String | Type))
                t = s.as?(Type)
                if t.nil?
                    g = @structs.fetch s, nil
                    return !g.nil?
                end
                return true
            end

            def compare_type(t : (Type | String), t1 : (Type | String)) : Bool
                if t == t1
                    return true
                end

                if t == Type::INT || t1 == Type::INT
                    return is_int(t) && is_int(t1)
                end

                return false
            end

            def is_int(t : (Type | String)) : Bool
                return t == Type::INT8 || t == Type::INT16 || t == Type::INT || t == Type::INT64
            end

        end
    end
end