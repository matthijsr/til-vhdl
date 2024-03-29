use criterion::{criterion_group, criterion_main, Criterion};
use til_parser::query::into_query_storage_default;

const TEST_STRING: &str = "namespace my::test::space {
    type byte = Bits(8);
    type select = Union(val: byte, empty: Null);
    type rgb = Group(r: select, g: select, b: select);
    type stream = Stream (
        data: rgb,
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
        user: Null,
        keep: false,
    );
    type stream2 = stream;

    #documentation#
    streamlet comp1 = (
        # some doc # a: in stream,
        b: out stream,
        c: in stream2,
        d: out stream2,
    );

    interface iface1 = (a: in stream, b: out stream);

    #streamlet documentation
bla bla bla#
    streamlet comp2 = iface1;

    #This is implementation
documentation, it can work as part of a declaration.#
    impl struct = (
        a: in stream, // Ports are declared as *name* : *direction* *stream name*
        b: out stream,
        c: in stream2,
        d: out stream2,
    ){
        // Ports can be connected with --
        a -- b;

        // Streamlet instances are declared with
        // *instance name* = *streamlet name*
        a = comp1;
        b = comp1;

        // Considering c, d = comp;

        // Ports on streamlet instances can be addressed with .
        a.a--b.b;
        a.b-- b.a;
        // As with everything else, whitespace does not matter
        
        // Ports on streamlet instances can also be connected to local ports
        c -- a.c;
        d -- b.d;
        a.d -- b.c;
    };

    streamlet comp3 = comp1 {
        impl:
        #This is implementation documentation, too.#
        {
            p1 = comp2;
            p2 = comp2;
            a -- p1.a;
            b -- p1.b;
            c -- p2.a;
            d -- p2.b;
        }
    };

    streamlet comp4 = comp1 {
        impl: struct
    };

    #This is a copy of an existing streamlet.
It will still be emitted, as the name has changed.
(And we're adding documentation).#
    streamlet comp5 = comp4;

    // Regardless, they can still be used as interfaces
    streamlet comp6 = comp5 {
        impl: struct,
    };

    // Final example, of the relative amount of work required to define a component:
    type my_stream = Stream(
        data: Bits(8),
        throughput: 5.0,
        dimensionality: 3,
        synchronicity: Sync,
        complexity: 7,
        direction: Forward, // Note, the keep and user properties are not required (nor is throughput, which defaults to 1)
    );
    #This is streamlet documentation
Newline.#
    streamlet my_new_streamlet = (
        in_port: in my_stream,
        #This is port documentation
Newline.#
        out_port: out my_stream,
    ) {
        impl: #This is impl documentation
Newline.# {
            in_port -- out_port;
        },
    };

    streamlet comp7 = comp4;

    type stream_base1 = Stream(
        data: Bits(8),
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
    );
    type stream_base2 = Stream(
        data: Bits(8),
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Reverse,
    );
    type multi_stream_group = Group(a: stream_base1, b: stream_base1, c: stream_base2);
    type multi_stream = Stream(
        data: multi_stream_group,
        throughput: 3.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
    );

    streamlet multi_streamlet = (x: in multi_stream, y: out multi_stream);
}";

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse namespace", |b| {
        b.iter(|| into_query_storage_default(TEST_STRING).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
