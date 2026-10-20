(component
  (type $io_err_ty
    (instance
      (export "error" (type (sub resource)))
    )
  )
  (import "wasi:io/error@0.2.0" (instance $io_error (type $io_err_ty)))
  (alias export $io_error "error" (type $io_error_res))

  (type $io_poll_ty
    (instance
      (export "pollable" (type (sub resource)))
    )
  )
  (import "wasi:io/poll@0.2.0" (instance $io_poll (type $io_poll_ty)))
  (alias export $io_poll "pollable" (type $pollable_res))

  (type $io_streams_ty
    (instance
      (alias outer 1 $io_error_res (type $err_outer))
      (export $err "error" (type (eq $err_outer)))
      (type $stream_error_def (variant (case "last-operation-failed" (own $err)) (case "closed")))
      (export $stream_error "stream-error" (type (eq $stream_error_def)))
      (export $input-stream "input-stream" (type (sub resource)))
      (export $output-stream "output-stream" (type (sub resource)))
      (export "[method]input-stream.read" (func (param "self" (borrow $input-stream)) (param "len" u64) (result (result (list u8) (error $stream_error)))))
      (export "[method]output-stream.blocking-write-and-flush" (func (param "self" (borrow $output-stream)) (param "contents" (list u8)) (result (result (error $stream_error)))))
    )
  )
  (import "wasi:io/streams@0.2.0" (instance $io_streams (type $io_streams_ty)))
  (alias export $io_streams "input-stream" (type $input_stream))
  (alias export $io_streams "output-stream" (type $output_stream))

  (type $stdin_ty
    (instance
      (alias outer 1 $input_stream (type $in_outer))
      (export $in "input-stream" (type (eq $in_outer)))
      (export "get-stdin" (func (result (own $in))))
    )
  )
  (import "wasi:cli/stdin@0.2.0" (instance $cli_stdin (type $stdin_ty)))

  (type $stdout_ty
    (instance
      (alias outer 1 $output_stream (type $out_outer))
      (export $out "output-stream" (type (eq $out_outer)))
      (export "get-stdout" (func (result (own $out))))
    )
  )
  (import "wasi:cli/stdout@0.2.0" (instance $cli_stdout (type $stdout_ty)))

  (type $stderr_ty
    (instance
      (alias outer 1 $output_stream (type $out_outer))
      (export $out "output-stream" (type (eq $out_outer)))
      (export "get-stderr" (func (result (own $out))))
    )
  )
  (import "wasi:cli/stderr@0.2.0" (instance $cli_stderr (type $stderr_ty)))

  (core module $mem_mod
    (memory (export "memory") 2)
    (global $bump (mut i32) (i32.const 65536))
    (func (export "cabi_realloc") (param $old_ptr i32) (param $old_size i32) (param $align i32) (param $new_size i32) (result i32)
      (local $ret i32)
      (local.set $ret (global.get $bump))
      (global.set $bump (i32.add (global.get $bump) (local.get $new_size)))
      (local.get $ret)
    )
  )
  (core instance $mem (instantiate $mem_mod))

  (alias export $cli_stdin "get-stdin" (func $get_stdin_f))
  (alias export $cli_stdout "get-stdout" (func $get_stdout_f))
  (alias export $cli_stderr "get-stderr" (func $get_stderr_f))
  (alias export $io_streams "[method]input-stream.read" (func $stream_read_f))
  (alias export $io_streams "[method]output-stream.blocking-write-and-flush" (func $stream_blocking_write_f))

  (core func $get_stdin (canon lower (func $get_stdin_f)))
  (core func $get_stdout (canon lower (func $get_stdout_f)))
  (core func $get_stderr (canon lower (func $get_stderr_f)))
  (core func $stream_read (canon lower (func $stream_read_f) (memory $mem "memory") (realloc (func $mem "cabi_realloc"))))
  (core func $stream_blocking_write (canon lower (func $stream_blocking_write_f) (memory $mem "memory")))

  (core module $app
    (import "env" "memory" (memory 2))
    (import "wasi" "get-stdin" (func $get_stdin (result i32)))
    (import "wasi" "get-stdout" (func $get_stdout (result i32)))
    (import "wasi" "get-stderr" (func $get_stderr (result i32)))
    (import "wasi" "read" (func $read (param i32 i64 i32)))
    (import "wasi" "blocking-write" (func $blocking_write (param i32 i32 i32 i32)))

    (func (export "echo-and-log") (param $max_len i32) (param $warn i32) (result i32)
      (local $in_h i32)
      (local $out_h i32)
      (local $err_h i32)
      (local $ptr i32)
      (local $len i32)

      (local.set $in_h (call $get_stdin))
      (call $read (local.get $in_h) (i64.extend_i32_u (local.get $max_len)) (i32.const 256))
      (if (i32.eq (i32.load (i32.const 256)) (i32.const 0))
        (then
          (local.set $ptr (i32.load (i32.const 260)))
          (local.set $len (i32.load (i32.const 264)))
          (if (i32.gt_u (local.get $len) (i32.const 0))
            (then
              (local.set $out_h (call $get_stdout))
              (call $blocking_write (local.get $out_h) (local.get $ptr) (local.get $len) (i32.const 64))
            )
          )
        )
        (else
          (local.set $len (i32.const 0))
        )
      )

      (if (i32.ne (local.get $warn) (i32.const 0))
        (then
          (local.set $err_h (call $get_stderr))
          (i32.store8 (i32.const 700) (i32.const 87))
          (i32.store8 (i32.const 701) (i32.const 65))
          (i32.store8 (i32.const 702) (i32.const 82))
          (i32.store8 (i32.const 703) (i32.const 78))
          (call $blocking_write (local.get $err_h) (i32.const 700) (i32.const 4) (i32.const 64))
        )
      )

      (local.get $len)
    )
  )

  (core instance $app_inst
    (instantiate $app
      (with "env" (instance $mem))
      (with "wasi" (instance
        (export "get-stdin" (func $get_stdin))
        (export "get-stdout" (func $get_stdout))
        (export "get-stderr" (func $get_stderr))
        (export "read" (func $stream_read))
        (export "blocking-write" (func $stream_blocking_write))
      ))
    )
  )

  (func $echo_and_log (param "max-len" s32) (param "warn" s32) (result s32)
    (canon lift (core func $app_inst "echo-and-log"))
  )
  (export "echo-and-log" (func $echo_and_log))
)
