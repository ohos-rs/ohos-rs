#!/bin/sh

ohpm_private_key = 'Ranger123.'

expect<<- END
set timeout 60
spawn ohpm publish /Users/ranger/Desktop/project/ohos-rs/packages/crates/snappy/package.har
expect {
"*private key:" { send " Ranger123.\n" }
}
expect eof
exit