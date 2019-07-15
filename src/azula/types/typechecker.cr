require "../ast/*"
require "../token"

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
            @errors : Array(String)

            getter errors

            def initialize
                @variables = {} of String=>(Type | String)
                @structs = {} of String=>Hash(String, (Type | String))
                @errors = [] of String
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
                        #return val
                    end
                    return [Type::VOID]
                when .is_a?(AST::Block)
                    convert_and_check_nil Block
                    node.statements.each do |stmt|
                        val = self.check stmt
                        return_if_nil val
                        if val.size == 0
                            next
                        end
                        return val
                    end
                    return [Type::VOID]
                when .is_a?(AST::IntegerLiteral)
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
                    return [node.struct_name.ident]
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
                    ident_types = [] of (Type | String)
                    node.idents.each do |val|
                        v = self.check val
                        return_if_nil v
                        ident_types = ident_types + v
                    end
                    if val_types.size != ident_types.size
                        self.add_error "incorrect number of values in assign statement", node.token
                        return [Type::VOID]
                    end
                    val_types.size.times do |i|
                        if val_types[i] != ident_types[i]
                            self.add_error "incorrect value in assign statement, trying to assign #{val_types[i]} to variable of type #{ident_types[i]}", node.token
                            return [Type::VOID]
                        end
                    end
                    node.idents.each do |val|
                        val = val.as?(AST::TypedIdentifier)
                        if val.nil?
                            next
                        end
                        @variables[val.ident] = val.type 
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
                else
                    return [Type::VOID]
                end
            end

            def add_error(s : String, token : Token)
                @errors << "#{s} (file #{token.file}, line #{token.linenumber}, char #{token.charnumber})"
            end

        end
    end
end