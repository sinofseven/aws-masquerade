aws-masquerade
===

CLI tool which enables you to login and retrieve [AWS](https://aws.amazon.com/) temporary credentials using with Assuming Role.

## Install
### From binaries
Check out the [Release page](https://github.com/sinofseven/aws-masquerade/releases) for prebuild versions of `aws-masquerade` for several different architectures.

### From source
```
cargo install aws-masquerade
```

## Usage
```
aws-masquerade 0.1.0
sinofseven
assume iam role

USAGE:
    aws-masquerade [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add            add a account
    assume         exec assume role
    config-path    show path of config file
    edit           edit a account
    help           Prints this message or the help of the given subcommand(s)
    list           list accounts
    remove         remove a account
    validate       validate config
    view           view a account
```

### `aws-masquerade add`: add account

To add a account to aws-masquerade just run the following command and follow the prompts.

```bash
$ aws-masquerade add
account name (required): account_name # account name (using for assumed profile name)
source profile name []: # source profile for assumimg role
role arn (required): arn:aws:iam::000000000000:role/target-role # target iam role arn for assumeing role
mfa arn []: arn:aws:iam::000000000000:mfa/user-name   # virtual mfa device arn (if using mfa)
mfa secret []: SDI7UGDNQ5NURIUPBOWEUTHIDBIT6DRHR4WLWS7N7C3C6VS3LJKNWHL2JZIFIUYI # secret of virtual mfa device 

Select Credential Output Type: # output format of assume role result
 [0] SharedCredentials # adding to shared config
 [1] bash # bash style. export AWS_ACCESS_KEY_ID="xxxxxxxxxxxx"
 [2] fish # fish style. set -x AWS_ACCESS_KEY_ID "xxxxxxxxxxxx"
 [3] PowerShell # PowerShell style. $env:AWS_ACCESS_KEY_ID="xxxxxxxxxxxx"

 > [0]: 

Select awscli output type: # the output from the AWS Command Line Interface (AWS CLI). 
 [0] json
 [1] text
 [2] table

 > []: 
Default Region Name []: ap-northeast-1 # set default region

Generated Account

{
  "test": {
    "sourceProfile": null,
    "roleArn": "arn:aws:iam::148005307600:role/aws-initialize-stack/administrator",
    "mfaArn": "arn:aws:iam::261267950596:mfa/yuta",
    "mfaSecret": "SDI7UGDNQ5NURIUPBOWEUTHIDBIT6DRHR4WLWS7N7C3C6VS3LJKNWHL2JZIFIUYI",
    "credentialOutput": "SharedCredentials",
    "output": null,
    "region": "ap-northeast-1"
  }
}

Do you confirm add account? (y/n) [y]: 
```

### `aws-masquerade assume -a account-name`: exec assume role
```bash
$ aws-masquerade assume --help
aws-masquerade-assume 
exec assume role

USAGE:
    aws-masquerade assume [OPTIONS] --account-name <account>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account-name <account>    Name of the account
    -t, --mfa-token <token>         Input Mfa Token
```

If you are using MFA, enter the MFA token optionally or interactively.  
MFA tokens are automatically populated if you have registered MFA secrets.  

#### Credential Output Type
##### CredentialOutput: SharedCredentials
The result of the Assume Role will be added to SharedConfig.

##### CredentialOutput: bash
```bash
$ aws-masquerade assume -a account-name
export AWS_ACCESS_KEY_ID="XXXXXXXXXXXXXXXXXXXX"
export AWS_SECRET_ACCESS_KEY="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
export AWS_SESSION_TOKEN="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
export AWS_SECURITY_TOKEN="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# Run this to configure your shell:
# eval $(aws-masquerade assume -a account-name)
```

##### CredentialOutput: fish
```fish
$ aws-masquerade assume -a account-name
set -gx AWS_ACCESS_KEY_ID "XXXXXXXXXXXXXXXXXXXX"
set -gx AWS_SECRET_ACCESS_KEY "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
set -gx AWS_SESSION_TOKEN "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
set -gx AWS_SECURITY_TOKEN "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# Run this to configure your shell:
# eval (aws-masquerade assume -a account-name)
```

##### CredentialOutput: PowerShell
```powershell
$ aws-masquerade assume -a account-name
$env:AWS_ACCESS_KEY_ID="XXXXXXXXXXXXXXXXXXXX"
$env:AWS_SECRET_ACCESS_KEY="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
$env:AWS_SESSION_TOKEN="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
$env:AWS_SECURITY_TOKEN="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# Run this to configure your shell:
# eval $(aws-masquerade assume -a account-name)
```


### `aws-masquerade view -a account-name`: view account configure
```bash
$ aws-masquerade view --help
aws-masquerade-view 
view a account

USAGE:
    aws-masquerade view --account-name <account>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account-name <account>    Name of the account


$ aws-masquerade -a account-name
{
  "account-name": {
    "sourceProfile": null,
    "roleArn": "arn:aws:iam::000000000000:role/xxxxxxxxxxx",
    "mfaArn": "arn:aws:iam::000000000000:mfa/xxxxxxxxxxx",
    "mfaSecret": "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "credentialOutput": "SharedCredentials",
    "output": null,
    "region": "ap-northeast-1"
  }
}
```

### `aws-masquerade list`: show account list
```bash
$ aws-masquerade list
account-001
account-002
account-003
```

### `aws-masquerade edit -a account-name`: edit a existing account
```bash
$ aws-masquerade edit --help
aws-masquerade-edit 
edit a account

USAGE:
    aws-masquerade edit --account-name <account>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account-name <account>    Name of the account
```

egit prompt is almost as same as add prompt.

### `aws-masquerade remove -a account-name`: remove a account
```bash
$ aws-masquerade remove --help
aws-masquerade-remove 
remove a account

USAGE:
    aws-masquerade remove --account-name <account>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account-name <account>    Name of the account
```

### `aws-masquerade config-path`: show config file path
```bash
$ aws-masquerade config-path
/home/codespace/.config/aws-masquerade/config.json
```

### `aws-masquerade validate`: validate config file
```
aws-masquerade validate
```

## Author
[sinofseven](https://github.com/sinofseven)