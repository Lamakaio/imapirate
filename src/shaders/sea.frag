#version 450
#ifdef GL_ES
precision mediump float;
#endif

layout(location=0)in vec2 v_Uv;
layout(location=0)out vec4 o_Target;
layout(set=1,binding=1)uniform SeaMaterial_time{
    float u_time;
    float u_zoom;
    vec2 u_offset;
};
vec3 wave(vec2 dir,vec2 intensity,float freq,vec2 pos,float speed){
    float dist=dot(dir,pos)+u_time*10.*speed;
    vec2 waves=vec2(cos(freq*dist),sin(freq*dist));
    mat2 der=mat2(0.,-freq,freq,0.);
    return vec3(dot(der*waves,intensity)*dir,dot(waves,intensity));
}

vec2 sea(vec2 pos){
    vec3 value=vec3(0.,0.,0.);//value and derivative
    float r1=.2;
    vec2 dir=vec2(2.*r1,2.-2.*r1);
    float norm=sqrt(dot(dir,dir));
    dir=dir/vec2(norm,norm);
    vec2 intensity=vec2(.5,.5);
    float freq=.3142;
    float speed=.5454;
    value+=wave(dir,intensity,freq,pos,speed);
    
    r1=.2;
    dir=vec2(2.*r1,2.-2.*r1);
    norm=sqrt(dot(dir,dir));
    dir=dir/vec2(norm,norm);
    intensity=vec2(.5,.5);
    freq=.512;
    speed=.7125;
    value+=wave(dir,intensity,freq,pos,speed);
    
    r1=.1325;
    dir=vec2(2.*r1,2.-2.*r1);
    norm=sqrt(dot(dir,dir));
    dir=dir/vec2(norm,norm);
    intensity=vec2(.17,.357);
    freq=.772;
    speed=.6351;
    value+=wave(dir,intensity,freq,pos,speed);
    
    r1=-.5168465;
    dir=vec2(2.*r1,2.-2.*r1);
    norm=sqrt(dot(dir,dir));
    dir=dir/vec2(norm,norm);
    intensity=vec2(.04,.04);
    freq=.712;
    speed=.19145;
    value+=wave(dir,intensity,freq,pos,speed);
    
    r1=.1832;
    dir=vec2(2.*r1,2.-2.*r1);
    norm=sqrt(dot(dir,dir));
    dir=dir/vec2(norm,norm);
    intensity=vec2(.3,.5);
    freq=.2475;
    speed=.9;
    value+=wave(dir,intensity,freq,pos,speed);
    
    return vec2(value.z,sqrt(dot(value.xy,value.xy)));
}

void main(){
    const vec3 darkblue=vec3(13./255.,16./255.,198./255.);
    const vec3 lightblue=vec3(71./255.,114./255.,255./255.);
    const vec3 white=vec3(211./255.,231./255.,255./255.);
    vec2 p=(floor(gl_FragCoord.xy/1.)*1.+u_offset)*u_zoom;
    
    vec2 s=sea(p);
    float height=s.x;
    float der=s.y;
    if(height>=1.5&&der>=-.1&&der<=.1){
        o_Target=vec4(white,1.);
    }
    else if(height>.5){
        o_Target=vec4(lightblue,1.);
    }
    else{
        o_Target=vec4(darkblue,1.);
    }
}