# the original idea is taken from github/hub completion file

# Copyright (c) 2009 Chris Wanstrath

# Permission is hereby granted, free of charge, to any person obtaining
# a copy of this software and associated documentation files (the
# "Software"), to deal in the Software without restriction, including
# without limitation the rights to use, copy, modify, merge, publish,
# distribute, sublicense, and/or sell copies of the Software, and to
# permit persons to whom the Software is furnished to do so, subject to
# the following conditions:

# The above copyright notice and this permission notice shall be
# included in all copies or substantial portions of the Software.

# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
# LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
# OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
# WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

_git 2>/dev/null

# stash the "real" command for later
functions[_gitext_orig_git_commands]=$functions[_git_commands]

# Replace it with our own wrapper.
declare -f _git_commands >& /dev/null && unfunction _git_commands
_git_commands () {
  local ret=1
  # call the original routine
  _call_function ret _gitext_orig_git_commands

  # Effectively "append" our hub commands to the behavior of the original
  # _git_commands function.  Using this wrapper function approach ensures
  # that we only offer the user the hub subcommands when the user is
  # actually trying to complete subcommands.
  gitext_commands=(
    browse:'browse the project in web UI'
  )
  _describe -t gitext-commands 'gitext command' gitext_commands && ret=0

  return ret
}

_gitext() {
  service=git
  declare -f _git >& /dev/null && _git
}

compdef _gitext gitext