// automatically generated by the FlatBuffers compiler, do not modify
export class StructInNestedNS {
    constructor() {
        this.bb = null;
        this.bb_pos = 0;
    }
    __init(i, bb) {
        this.bb_pos = i;
        this.bb = bb;
        return this;
    }
    a() {
        return this.bb.readInt32(this.bb_pos);
    }
    mutate_a(value) {
        this.bb.writeInt32(this.bb_pos + 0, value);
        return true;
    }
    b() {
        return this.bb.readInt32(this.bb_pos + 4);
    }
    mutate_b(value) {
        this.bb.writeInt32(this.bb_pos + 4, value);
        return true;
    }
    static getFullyQualifiedName() {
        return 'NamespaceA.NamespaceB.StructInNestedNS';
    }
    static sizeOf() {
        return 8;
    }
    static createStructInNestedNS(builder, a, b) {
        builder.prep(4, 8);
        builder.writeInt32(b);
        builder.writeInt32(a);
        return builder.offset();
    }
    unpack() {
        return new StructInNestedNST(this.a(), this.b());
    }
    unpackTo(_o) {
        _o.a = this.a();
        _o.b = this.b();
    }
}
export class StructInNestedNST {
    constructor(a = 0, b = 0) {
        this.a = a;
        this.b = b;
    }
    pack(builder) {
        return StructInNestedNS.createStructInNestedNS(builder, this.a, this.b);
    }
}
