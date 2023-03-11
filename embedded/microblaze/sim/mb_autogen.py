# 
#  Scratchpad MMU - Autogenerator for some Microblaze opcodes
#  Copyright (C) Jack Whitham 2009
#  http://www.jwhitham.org.uk/c/smmu.html
# 
#  This library is free software; you can redistribute it and/or
#  modify it under the terms of the GNU Lesser General Public
#  License as published by the Free Software Foundation
#  (version 2.1 of the License only).
#  
#  This library is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
#  Lesser General Public License for more details.
#  
#  You should have received a copy of the GNU Lesser General Public
#  License along with this library; if not, write to the Free Software
#  Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA
# 

import re

DEBUG = False

def Get_Cmp(topcode):
    assert topcode == 5
    return """
    switch ( iword & 3 )
    {
        case 1 : /* cmp */
            out &= ~ ( 1 << 31 ) ;
            if ( (int) a > (int) b ) out |= 1 << 31 ;
            snprintf ( name , NS , "cmp: r%u = r%u > r%u = %u\\n" ,
                    rD , rA , rB , ((unsigned) out ) >> 31 ) ;
            break ;
        case 3 : /* cmpu */
            memcpy ( name , "cmpu " , 5 ) ;
            out &= ~ ( 1 << 31 ) ;
            if ( a > b ) out |= 1 << 31 ;
            snprintf ( name , NS , "cmpu: r%u = r%u > r%u = %u\\n" ,
                    rD , rA , rB , ((unsigned) out ) >> 31 ) ;
            break ;
        default :/* rsubk */
            break ;
    }
"""

def Get_Pcmp_Eq_Ne(topcode):
    
    if ( topcode == 0x22 ):
        function = "eq"
        function1 = "=="
    else:
        assert topcode == 0x23
        function = "ne"
        function1 = "!="

    return ("""
if ( iword & 0x400 )
{
    out = ( a """ + function1 + """ b ) ;
    snprintf ( name , NS , "pcmp""" + function + ": r%u " + function1 +
        """ r%u = %u\\n" , rA , rB , out ) ;
}
""")

def Get_Pcmp_Bf(topcode):
    assert topcode == 0x20
    out = []
    out.append("if ( iword & 0x400 ) { temp = 0 ;\n")
    for i in range(4):
        out.append("if ((( a >> %u ) & 0xff ) == (( b >> %u ) & 0xff ))" % 
                    (i * 8, i * 8))
        out.append("{ temp = %u ; }\n" % (4 - i))
    out.append("""
    snprintf ( name , NS , "pcmpbf: r%u versus r%u = %u\\n" , 
                rA , rB , temp ) ;
    out = temp ;
}
""")
    return ''.join(out)



def Get_ALU():
    SPECIAL_CASE = {
            0x5 : Get_Cmp ,
            0x20 : Get_Pcmp_Bf,
            0x22 : Get_Pcmp_Eq_Ne,
            0x23 : Get_Pcmp_Eq_Ne,
            }

    REGULAR_ALU_OPS = [
        ('add', '', 0x0, (False, True, '+', False, False)),
        ('add', 'c', 0x2, (True, True, '+', False, False)),
        ('add', 'k', 0x4, (False, False, '+', False, False)),
        ('add', 'kc', 0x6, (True, False, '+', False, False)),
        ('and', '', 0x21, (False, False, '&', False, False)),
        ('andn', '', 0x23, (False, False, '&', False, True)),
        ('xor', '', 0x22, (False, False, '^', False, False)),
        ('or', '', 0x20, (False, False, '|', False, False)),
        ('rsub', '', 0x1, (False, True, '+', True, False)),
        ('rsub', 'c', 0x3, (True, True, '+', True, False)),
        ('rsub', 'k', 0x5, (False, False, '+', True, False)),
        ('rsub', 'kc', 0x7, (True, False, '+', True, False)),
        ('mul', '', 0x10, (False, False, '*', False, False)) ]

    out = []
    for (mne1, mne2, topcode0, (carry_in_enable, carry_out_enable, 
            function, invert_a, invert_b)) in REGULAR_ALU_OPS:

        for topcode in (topcode0, topcode0 | 0x8):
            out.append('\ncase 0x%x :\n{' % topcode)

            c_function = function
            subtract = invert_a and ( function == '+' )
            use_64_bit = carry_out_enable and ( function == '+' )

            cin = 0 # int(subtract)

            if ( use_64_bit ):
                out.append('  uint64_t out ;\n')
                expander = '(uint64_t)'
            else:
                out.append('  unsigned out ;\n')
                expander = ''

            if ( carry_in_enable ):
                out.append('  carry_in = %u ;\n' % cin)
                out.append('  if (( c -> msr & MSR_C ) != 0 ) '
                                    'carry_in = %u ;\n' % (1 - cin))

            if function == "*":
                out.append('bubble = 2;\n')

            out.append('  out = ( %s ' % expander)
            if ( invert_b ):
                out.append('( ~ b ) ')
            else:
                out.append('b ')

            if ( invert_a ):
                out.append("%s ( ~ ( %s a )) " % (function, expander))
            else:
                out.append("%s ( %s a ) " % (function, expander))

            if ( carry_in_enable ):
                out.append(' %s carry_in' % function)
            elif ( subtract ):
                out.append(' %s 1' % function)

            if ( subtract ):
                c_function = '-'

            out.append(' ) ;\n')

            if ( carry_out_enable ):
                out.append('  c -> msr &= ~MSR_C ;\n') # carry cleared
                out.append('  if ( ')
                if ( subtract ):
                    out.append('! ')
                out.append('(( out & ( (uint64_t) 1 ' +
                        '<< (uint64_t) 32 )) != 0 )) {')
                out.append('\n    c -> msr |= MSR_C ;\n')
                out.append('  }\n')

            out.append('  snprintf ( name , NS , "' + mne1 + mne2 + 
                    ': r%u =')
            if ( topcode & 0x8 ):
                out.append(' 0x%x ')
            else:
                out.append(' r%u ')

            out.append(c_function + ' r%u')

            if ( carry_in_enable ):
                out.append(' (cin)')
                
            out.append(' = 0x%x')
            if ( carry_out_enable ):
                out.append(' (cout=%u)')

            out.append('\\n" , rD , ')

            if ( topcode & 0x8 ):
                out.append('b , ')
            else:
                out.append('rB , ')

            out.append('rA , (unsigned) out ')
            if ( carry_out_enable ):
                out.append(', !! ( c -> msr & MSR_C )')

            out.append(') ;')

            sc = SPECIAL_CASE.get(topcode, None)
            if ( sc != None ):
                out.append(sc(topcode))


            if (( mne1 == 'or' ) and ( mne2 == '' )):
                # Special case - check for simulator command
                out.append("""
        {
            unsigned cmd = iword & 0x7ff ;

            if (( iword & ~0x7ff ) == MB_NOP )
            {
                if ( cmd != 0 )
                {
                    snprintf ( name , NS , "cmd: 0x%x\\n" , iword & 0x7ff ) ;
                    c -> trace_fn ( c -> t_user , 
                            c , MB_SIM_CMD , IP ( cmd )) ;
                    latency = 0 ;
                } else {
                    snprintf ( name , NS , "nop:\\n" ) ;
                }
            }
        }\n""")

            if ( topcode == 0 ):
                out.append("""
        if ( iword == 0 )
        {
            c -> trace_fn ( c -> t_user , c , 
                        MB_ILLEGAL_INST , IP ( c -> pc )) ;
            valid = 0 ;
        }""")

            out.append('  Set_D ( c , (unsigned) out ) ;\n')
            out.append('  } break ;\n')

    return ''.join(out)

def Get_LSU():
    LOAD_STORE_OPS = [
        ((True, 4), "sw", 2),
        ((True, 2), "sh", 1),
        ((True, 1), "sb", 0),
        ((False, 4), "lwu", 2),
        ((False, 2), "lhu", 1),
        ((False, 1), "lbu", 0) ]

    out = []
    for ((store, size), mne, topcode_part) in LOAD_STORE_OPS:
        topcode = topcode_part 
        if ( store ):
            topcode |= 0x34 # store code
        else:
            topcode |= 0x30 # load code

        out.append('\ncase 0x%x :\n'
                    'case 0x%x : /* LSU: %s */\n' % (topcode, 
                                topcode | 0x8, mne) )
        out.append('  ea = a + b ;\n')
        if ( size == 4 ):
            out.append('  if ( ea & 3 ) '
            'c -> trace_fn ( c -> t_user , c , MB_ALIGN , IP ( ea )) ;\n')
        elif ( size == 2 ):
            out.append('  if ( ea & 1 ) '
            'c -> trace_fn ( c -> t_user , c , MB_ALIGN , IP ( ea )) ;\n')

        if ( store ):
            out.append('c -> trace_fn ( c -> t_user , c , '
                            'MB_STORE , IP ( ea )) ;\n')
            out.append('  c -> store_fn ( c -> m_user , ea , '
                            'Get_D ( c ) , %u ) ;\n' % size )
            out.append('  snprintf ( name , NS , "' + mne + 
                ': [ 0x%x ] = r%u = 0x%x\\n" , ea , rD , Get_D ( c ) ) ;\n')
        else:
            out.append('bubble = 2;\n')
            out.append('c -> trace_fn ( c -> t_user , c , '
                            'MB_LOAD , IP ( ea )) ;\n')
            out.append('  Set_D ( c , '
                'c -> load_fn ( c -> m_user , ea , %u ) ) ;\n' % size)
            out.append('  snprintf ( name , NS , "' + mne + 
                ': r%u = [ 0x%x ] = 0x%x\\n" , rD , ea , Get_D ( c ) ) ;\n')

        out.append('  break ;\n')

    return ''.join(out)

def Get_Barrel():
    BARREL_OPS = [ 
        ('bsrl', 0, '>>', False), 
        ('bsra', 1, '>>', True), 
        ('bsll', 2, '<<', False) ]

    out = []
    for topcode in (0x11, 0x19):
        out.append("\ncase 0x%x :\n" % topcode)
        out.append("switch ( iword & ( 3 << 9 )) {\n")

        for (mne, opcode, java_op, special) in BARREL_OPS:
            out.append('  case 0x%x : /* Barrel: %s */\n' % (opcode << 9, mne) )
            out.append('   bubble = 1;\n')
            out.append('   b = b & 0x1f ;\n')
            if ( special ):
                # special means sign extend
                out.append('    Set_D ( c , ((int) a) %s b ) ;\n' % java_op)
            else:
                out.append('    Set_D ( c , a %s b ) ;\n' % java_op)

            out.append('  snprintf ( name , NS , "' + mne + 
                ': r%u = r%u ' + java_op)
            if ( topcode & 0x8 ):
                out.append(' 0x%x')
            else:
                out.append(' r%u')

            out.append(' = 0x%x\\n" , rD , rA ,')
            if ( topcode & 0x8 ):
                out.append(' b , ')
            else:
                out.append(' rB , ')

            out.append('Get_D ( c ) ) ;\n')
            out.append('    break ;\n')
        out.append('    default : Set_D ( c , 0 ) ; break ;\n' )
        out.append("""  }
      rB = 0;
      break ;\n""")

    return ''.join(out)

def Get_Pycode():
    out = []
    out.append(Get_ALU())
    out.append(Get_LSU())
    out.append(Get_Barrel())
    return ''.join(out)

def Main():
    fout = open('mb_autogen.c', 'wt')
    fout.write(Get_Pycode())
    fout.close()

if ( __name__ == "__main__" ):
    Main()

