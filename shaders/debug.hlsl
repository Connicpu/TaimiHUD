struct VSInput
{
    float3 position: POSITION;
    float3 normal: NORMAL;
    float3 color: COLOR0;
    column_major matrix Model: MODEL;
    float3 colour: COLOUR;
     uint        instId  : SV_InstanceID;
};

cbuffer ConstantBuffer : register(b0)
{
  column_major matrix View;
  column_major matrix Projection;
}

struct VSOutput
{
    float4 position: SV_Position;
    float3 normal: NORMAL;
    float3 color: COLOR0;
    float3 colour: COLOUR;
};

VSOutput VSMain(VSInput input)
{
    VSOutput output = (VSOutput)0;

    float4 VertPos = float4(input.position, 1.0);
    output.position = mul(input.Model, VertPos);
    output.position = mul(View, output.position);
    output.position = mul(Projection, output.position);

    output.normal = input.normal;
    output.color = input.color;
    output.colour = input.colour;

    return output;
}


struct PSInput
{
    float4 position: SV_Position;
    float3 normal: NORMAL;
    float3 color: COLOR0;
    float3 colour: COLOUR;
};

struct PSOutput
{
    float4 color: SV_Target0;
};

PSOutput PSMain(PSInput input)
{
    PSOutput output = (PSOutput)0;
    output.color = float4(input.position.z, input.position.w, 1.0, 1.0);
    //output.color = float4(input.color * input.colour, 1.0);
    return output;
}
