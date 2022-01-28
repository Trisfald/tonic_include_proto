# Tonic include_proto utilities

A crate to make using `tonic::include_proto` less painful.

## namespaced!

This macro invokes the macro `tonic::include_proto` for multiple protobuf packages
and each of them are placed in the correct namespace.

### Example

The code:
```
tonic_include_proto::namespaced!("x.y", "x.z");
```
is equivalent to
```
mod x {
    mod y {
        tonic::include_proto!("x.y");
    }
    mod z {
        tonic::include_proto!("x.z");
    }
}
```
