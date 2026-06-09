### uuid



#### 方法

*  generate_uuid_str(char *out_uuid)

-  参数

```
out_uuid: 生成的 UUID 字符串，长度 36
```

-  返回值

```
int: 0 表示成功，-1 表示失败
```

-  描述

```
生成一个 UUID 字符串，并将其写入到 out_uuid 中
```


*  check_uuid_dir(char *uuid)

-  参数

```
uuid: UUID 字符串
```
-  返回值

```
int: 0 表示成功，-1 表示失败
```

-  描述

```
检查 UUID 对应的目录是否存在
```
